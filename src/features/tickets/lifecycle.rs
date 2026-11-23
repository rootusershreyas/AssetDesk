use super::model::{Priority, ReviewState, StartWorkError, TicketStatus, start_work};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum PersistStartError {
    TicketNotFound,
    Rule(StartWorkError),
    IneligibleHandler,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for PersistStartError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// The actor is supplied by the trusted caller. Return the committed version.
pub async fn start_assigned_ticket(
    pool: &PgPool,
    ticket_id: Uuid,
    actor: Uuid,
    expected_version: i64,
) -> Result<i64, PersistStartError> {
    let mut tx = pool.begin().await?;
    let ticket = lock_ticket(&mut tx, ticket_id)
        .await?
        .ok_or(PersistStartError::TicketNotFound)?;
    let state = ticket.review_state();
    let next_status = start_work(
        &state,
        expected_version,
        ticket.assignee_id.as_ref(),
        &actor,
    )
    .map_err(PersistStartError::Rule)?;

    if !handler_is_eligible(&mut tx, &ticket, actor).await? {
        return Err(PersistStartError::IneligibleHandler);
    }
    let version = sqlx::query_scalar(
        "UPDATE tickets SET status=$2, version=version+1 WHERE id=$1 RETURNING version",
    )
    .bind(ticket_id)
    .bind(next_status)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO ticket_events(ticket_id, actor_id, event_type, resulting_version, old_status, new_status)
                 VALUES($1,$2,'started',$3,$4,$5)")
        .bind(ticket_id).bind(actor).bind(version).bind(ticket.status).bind(next_status)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(version)
}

#[derive(sqlx::FromRow)]
struct LockedTicket {
    priority: Priority,
    version: i64,
    status: TicketStatus,
    assignee_id: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    stamped: bool,
    category_id: Uuid,
}
impl LockedTicket {
    fn review_state(&self) -> ReviewState {
        ReviewState {
            priority: self.priority,
            version: self.version,
            status: self.status,
            has_assignee: self.assignee_id.is_some(),
            has_reviewer: self.reviewed_by.is_some(),
            has_review_timestamp: self.stamped,
        }
    }
}
async fn lock_ticket(
    conn: &mut sqlx::PgConnection,
    id: Uuid,
) -> Result<Option<LockedTicket>, sqlx::Error> {
    sqlx::query_as(
        "SELECT priority, version, status, assignee_id, reviewed_by,
        reviewed_at IS NOT NULL AS stamped, category_id FROM tickets WHERE id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(conn)
    .await
}
async fn handler_is_eligible(
    conn: &mut sqlx::PgConnection,
    ticket: &LockedTicket,
    actor: Uuid,
) -> Result<bool, sqlx::Error> {
    Ok(match ticket.priority {
        Priority::P1 => ticket.reviewed_by == Some(actor) && sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM users WHERE id=$1 AND role='it_manager' AND account_status='active' FOR SHARE"
        ).bind(actor).fetch_optional(&mut *conn).await?.is_some(),
        _ => sqlx::query_scalar::<_, Uuid>(
            "SELECT u.id FROM users u JOIN category_agents a ON a.user_id=u.id
             WHERE u.id=$1 AND a.category_id=$2 AND u.role='it_member'
             AND u.account_status='active' FOR SHARE OF u, a"
        ).bind(actor).bind(ticket.category_id).fetch_optional(&mut *conn).await?.is_some(),
    })
}

#[derive(Debug)]
pub enum PersistResolutionError {
    TicketNotFound,
    Rule(super::service::SubmitResolutionError),
    IneligibleHandler,
    Database(sqlx::Error),
}
impl From<sqlx::Error> for PersistResolutionError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

pub async fn resolve_assigned_ticket(
    pool: &PgPool,
    ticket_id: Uuid,
    actor: Uuid,
    expected_version: i64,
    resolution: &str,
) -> Result<i64, PersistResolutionError> {
    let mut tx = pool.begin().await?;
    let ticket = lock_ticket(&mut tx, ticket_id)
        .await?
        .ok_or(PersistResolutionError::TicketNotFound)?;
    let submitted = super::service::submit_resolution(
        &ticket.review_state(),
        expected_version,
        ticket.assignee_id.as_ref(),
        &actor,
        resolution,
    )
    .map_err(PersistResolutionError::Rule)?;
    if !handler_is_eligible(&mut tx, &ticket, actor).await? {
        return Err(PersistResolutionError::IneligibleHandler);
    }
    let version = sqlx::query_scalar(
        "UPDATE tickets SET resolution=$2, status=$3, version=version+1 WHERE id=$1 RETURNING version")
        .bind(ticket_id).bind(submitted.resolution).bind(submitted.status).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO ticket_events(ticket_id, actor_id, event_type, resulting_version, old_status, new_status, resolution)
        VALUES($1,$2,'resolution_submitted',$3,$4,$5,$6)")
        .bind(ticket_id).bind(actor).bind(version).bind(ticket.status).bind(submitted.status).bind(submitted.resolution)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(version)
}
