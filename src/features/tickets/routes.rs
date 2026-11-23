use super::{
    dto::{CreateInputError, CreateTicketInput},
    repository::{CreateTicketError, create_ticket},
    service::CreationError,
    validation::ValidationError,
};
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// HTTP adapter for use after the route extracts the trusted requester and JSON body.
pub async fn create_ticket_response(
    pool: &PgPool,
    requester: Uuid,
    input: &CreateTicketInput<Uuid>,
) -> Response {
    match create_ticket(pool, requester, input).await {
        Ok(id) => (
            StatusCode::CREATED,
            [(header::LOCATION, format!("/api/v1/tickets/{id}"))],
            Json(json!({ "id": id })),
        )
            .into_response(),
        Err(error) => {
            let (status, code, message) = match error {
                CreateTicketError::Validation(CreationError::CategoryNotFound) => (
                    StatusCode::NOT_FOUND,
                    "category_not_found",
                    "Category was not found.",
                ),
                CreateTicketError::Validation(CreationError::CategoryInactive) => (
                    StatusCode::CONFLICT,
                    "category_inactive",
                    "Choose an active category.",
                ),
                CreateTicketError::Validation(CreationError::CategoryTypeMismatch) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "validation_error",
                    "Category must match ticket type.",
                ),
                CreateTicketError::Validation(CreationError::Input(reason)) => {
                    let message = match reason {
                        CreateInputError::MissingCustomConcern => {
                            "Provide a custom concern for Incident Others."
                        }
                        CreateInputError::InvalidText(ValidationError::InvalidSubject) => {
                            "Subject must contain 1 to 160 characters."
                        }
                        CreateInputError::InvalidText(ValidationError::InvalidDescription) => {
                            "Description must contain 1 to 1000 characters."
                        }
                        CreateInputError::InvalidText(_) => "Invalid ticket text.",
                    };
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "validation_error",
                        message,
                    )
                }
                CreateTicketError::Database(_)
                | CreateTicketError::Validation(CreationError::CategoryLookupFailed) => {
                    tracing::error!("Ticket creation failed");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_error",
                        "Could not create ticket.",
                    )
                }
            };
            (
                status,
                Json(json!({"error": {"code": code, "message": message}})),
            )
                .into_response()
        }
    }
}

/// Supply the requester UUID from the trusted actor context.
pub async fn get_ticket_response(pool: &PgPool, requester: Uuid, ticket_id: Uuid) -> Response {
    match super::repository::find_ticket_for_requester(pool, ticket_id, requester).await {
        Ok(Some(ticket)) => Json(ticket).into_response(),
        Ok(None) => ticket_not_found(),
        Err(_) => {
            tracing::error!("Ticket lookup failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": {"code": "internal_error", "message": "Could not retrieve ticket."}})),
            ).into_response()
        }
    }
}

fn ticket_not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": {"code": "ticket_not_found", "message": "Ticket was not found."}})),
    )
        .into_response()
}

/// Supply the requester UUID from the trusted actor context.
pub async fn get_ticket_events_response(
    pool: &PgPool,
    requester: Uuid,
    ticket_id: Uuid,
) -> Response {
    match super::repository::list_events_for_requester(pool, ticket_id, requester).await {
        // Every committed ticket has a version-1 creation event.
        Ok(events) if events.is_empty() => ticket_not_found(),
        Ok(events) => Json(events).into_response(),
        Err(_) => {
            tracing::error!("Ticket history lookup failed");
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(json!({"error": {"code": "internal_error", "message": "Could not retrieve ticket history."}}))).into_response()
        }
    }
}
