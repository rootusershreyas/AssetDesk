use super::model::{Priority, ReviewError, ReviewState, TicketStatus, review_ticket};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum PersistReviewError {
    ManagerRequired,
    TicketNotFound,
    Review(ReviewError),
    Database(sqlx::Error),
}

impl From<sqlx::Error> for PersistReviewError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// manager_id comes from the actor context. Return the committed ticket version.
pub async fn review_and_assign(
    pool: &PgPool,
    ticket_id: Uuid,
    manager_id: Uuid,
    requested_assignee: Option<Uuid>,
    expected_version: i64,
) -> Result<i64, PersistReviewError> {
    let mut tx = pool.begin().await?;
    let manager = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE id=$1 AND role='it_manager' AND account_status='active' FOR SHARE"
    ).bind(manager_id).fetch_optional(&mut *tx).await?;
    if manager.is_none() {
        return Err(PersistReviewError::ManagerRequired);
    }

    let (priority, version, status, assigned, reviewed, stamped, category) =
        sqlx::query_as::<_, (Priority, i64, TicketStatus, bool, bool, bool, Uuid)>(
            "SELECT priority, version, status, assignee_id IS NOT NULL, reviewed_by IS NOT NULL,
                    reviewed_at IS NOT NULL, category_id FROM tickets WHERE id=$1 FOR UPDATE",
        )
        .bind(ticket_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(PersistReviewError::TicketNotFound)?;
    let state = ReviewState {
        priority,
        version,
        status,
        has_assignee: assigned,
        has_reviewer: reviewed,
        has_review_timestamp: stamped,
    };
    let eligible = match (priority, requested_assignee) {
        (Priority::P1, _) | (_, None) => None,
        (_, Some(id)) => {
            sqlx::query_scalar::<_, Uuid>(
                "SELECT u.id FROM users u JOIN category_agents a ON a.user_id=u.id
             WHERE u.id=$1 AND a.category_id=$2 AND u.role='it_member'
               AND u.account_status='active' FOR SHARE OF u, a",
            )
            .bind(id)
            .bind(category)
            .fetch_optional(&mut *tx)
            .await?
        }
    };
    let assignee = review_ticket(
        &state,
        expected_version,
        manager_id,
        requested_assignee,
        |id| Some(*id) == eligible,
    )
    .map_err(PersistReviewError::Review)?;
    let next_version = sqlx::query_scalar::<_, i64>(
        "UPDATE tickets SET assignee_id=$2, reviewed_by=$3, reviewed_at=now(), version=version+1
         WHERE id=$1 RETURNING version",
    )
    .bind(ticket_id)
    .bind(assignee)
    .bind(manager_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO ticket_events(ticket_id,actor_id,event_type,resulting_version,new_assignee_id,new_priority)
         VALUES($1,$2,'reviewed_and_assigned',$3,$4,$5)"
    ).bind(ticket_id).bind(manager_id).bind(next_version).bind(assignee).bind(priority)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(next_version)
}
