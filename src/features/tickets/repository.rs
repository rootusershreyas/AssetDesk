use super::{
    dto::CreateTicketInput,
    model::{Priority, TicketStatus},
    service::{CreationCategory, CreationError, validate_creation},
};
use crate::features::categories::model::TicketType;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum CreateTicketError {
    Validation(CreationError),
    Database(sqlx::Error),
}

impl From<sqlx::Error> for CreateTicketError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// requester_id is supplied by the trusted actor context.
pub async fn create_ticket(
    pool: &PgPool,
    requester_id: Uuid,
    input: &CreateTicketInput<Uuid>,
) -> Result<Uuid, CreateTicketError> {
    let mut tx = pool.begin().await?;
    let category = sqlx::query_as::<_, (String, TicketType, bool)>(
        "SELECT name, ticket_type, is_active FROM categories WHERE id = $1 FOR SHARE",
    )
    .bind(input.category_id)
    .fetch_optional(&mut *tx)
    .await?;
    validate_creation(input, |_| {
        category.map(|(name, ticket_type, is_active)| CreationCategory {
            name,
            ticket_type,
            is_active,
        })
    })
    .map_err(CreateTicketError::Validation)?;

    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO tickets (requester_id, category_id, ticket_type, priority,
            subject, description, custom_concern, ticket_number, status, version)
         VALUES ($1, $2, $3, $4, $5, $6, $7, CASE WHEN $3 = 'incident'
            THEN 'INC' || nextval('incident_number') ELSE 'SR' || nextval('request_number') END,
            'open', 1) RETURNING id",
    )
    .bind(requester_id)
    .bind(input.category_id)
    .bind(&input.ticket_type)
    .bind(input.priority)
    .bind(input.subject.trim())
    .bind(input.description.trim())
    .bind(input.custom_concern.as_deref().map(str::trim))
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO ticket_events (ticket_id, actor_id, event_type, resulting_version)
                 VALUES ($1, $2, 'created', 1)",
    )
    .bind(id)
    .bind(requester_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}

#[derive(Debug, sqlx::FromRow)]
pub struct TicketDetails {
    pub id: Uuid,
    pub ticket_number: String,
    pub requester_id: Uuid,
    pub category_id: Uuid,
    pub ticket_type: TicketType,
    pub priority: String,
    pub subject: String,
    pub description: String,
    pub custom_concern: Option<String>,
    pub status: String,
    pub version: i64,
}

/// Supply the requester ID from the actor context. Missing and invisible tickets both return None.
pub async fn find_ticket_for_requester(
    pool: &PgPool,
    ticket_id: Uuid,
    requester_id: Uuid,
) -> Result<Option<TicketDetails>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, ticket_number, requester_id, category_id, ticket_type, priority,
                subject, description, custom_concern, status, version
         FROM tickets WHERE id = $1 AND requester_id = $2",
    )
    .bind(ticket_id)
    .bind(requester_id)
    .fetch_optional(pool)
    .await
}

#[derive(Debug)]
pub enum ListTicketsError {
    InvalidLimit,
    InvalidOffset,
    Database(sqlx::Error),
}

pub async fn list_tickets_for_requester(
    pool: &PgPool,
    requester_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<TicketDetails>, ListTicketsError> {
    list_filtered_tickets_for_requester(pool, requester_id, None, None, limit, offset).await
}

pub async fn list_filtered_tickets_for_requester(
    pool: &PgPool,
    requester_id: Uuid,
    status: Option<TicketStatus>,
    priority: Option<Priority>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<TicketDetails>, ListTicketsError> {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);
    if !(1..=100).contains(&limit) {
        return Err(ListTicketsError::InvalidLimit);
    }
    if offset < 0 {
        return Err(ListTicketsError::InvalidOffset);
    }
    sqlx::query_as(
        "SELECT id, ticket_number, requester_id, category_id, ticket_type, priority,
                subject, description, custom_concern, status, version
         FROM tickets WHERE requester_id = $1
           AND ($4::text IS NULL OR status = $4)
           AND ($5::text IS NULL OR priority = $5)
         ORDER BY created_at DESC, id ASC LIMIT $2 OFFSET $3",
    )
    .bind(requester_id)
    .bind(limit)
    .bind(offset)
    .bind(status)
    .bind(priority)
    .fetch_all(pool)
    .await
    .map_err(ListTicketsError::Database)
}
