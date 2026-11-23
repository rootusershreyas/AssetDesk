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
    let (priority, version, status, assignee, reviewer, stamped, category) = sqlx::query_as::<
        _,
        (
            Priority,
            i64,
            TicketStatus,
            Option<Uuid>,
            Option<Uuid>,
            bool,
            Uuid,
        ),
    >(
        "SELECT priority, version, status, assignee_id, reviewed_by,
             reviewed_at IS NOT NULL, category_id FROM tickets WHERE id=$1 FOR UPDATE",
    )
    .bind(ticket_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(PersistStartError::TicketNotFound)?;
    let state = ReviewState {
        priority,
        version,
        status,
        has_assignee: assignee.is_some(),
        has_reviewer: reviewer.is_some(),
        has_review_timestamp: stamped,
    };
    let next_status = start_work(&state, expected_version, assignee.as_ref(), &actor)
        .map_err(PersistStartError::Rule)?;

    let eligible = match priority {
        Priority::P1 => reviewer == Some(actor) && sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM users WHERE id=$1 AND role='it_manager' AND account_status='active' FOR SHARE"
        ).bind(actor).fetch_optional(&mut *tx).await?.is_some(),
        _ => sqlx::query_scalar::<_, Uuid>(
            "SELECT u.id FROM users u JOIN category_agents a ON a.user_id=u.id
             WHERE u.id=$1 AND a.category_id=$2 AND u.role='it_member'
             AND u.account_status='active' FOR SHARE OF u, a"
        ).bind(actor).bind(category).fetch_optional(&mut *tx).await?.is_some(),
    };
    if !eligible {
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
        .bind(ticket_id).bind(actor).bind(version).bind(status).bind(next_status)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(version)
}
