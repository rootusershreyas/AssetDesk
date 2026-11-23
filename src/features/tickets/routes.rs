use super::{
    dto::{CreateInputError, CreateTicketInput, ListTicketsQuery, ReviewTicketInput},
    model::{AssignmentError, ReviewError},
    repository::{CreateTicketError, ListTicketsError, create_ticket},
    review::{PersistReviewError, review_and_assign},
    service::{CreationError, requester_ticket_page},
    validation::ValidationError,
};
use axum::{
    Json,
    extract::{Query, rejection::QueryRejection},
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

/// Return a ticket page for the requester supplied by the trusted actor context.
pub async fn list_tickets_response(
    pool: &PgPool,
    requester: Uuid,
    query: Result<Query<ListTicketsQuery>, QueryRejection>,
) -> Response {
    let (status, code, message) = match query {
        Err(_) => (
            StatusCode::BAD_REQUEST,
            "invalid_query",
            "Provide valid ticket filters and pagination parameters.",
        ),
        Ok(Query(query)) => match requester_ticket_page(pool, requester, query).await {
            Ok(page) => return Json(page).into_response(),
            Err(ListTicketsError::InvalidLimit) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                "Limit must be between 1 and 100.",
            ),
            Err(ListTicketsError::InvalidOffset) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                "Offset must be nonnegative.",
            ),
            Err(ListTicketsError::Database(_)) => {
                tracing::error!("Ticket listing failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Could not retrieve tickets.",
                )
            }
        },
    };
    (status, Json(json!({"error": {"code": code, "message": message}}))).into_response()
}

/// Supply the acting manager's identity from the trusted actor context.
pub async fn review_ticket_response(
    pool: &PgPool,
    actor: Uuid,
    ticket_id: Uuid,
    input: ReviewTicketInput,
) -> Response {
    use PersistReviewError::Review;
    if input.expected_version < 1 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "validation_error", "message": "Expected version must be positive."}})),
        ).into_response();
    }

    let result = review_and_assign(pool, ticket_id, actor, input.assignee_id, input.expected_version).await;
    let (status, code, message) = match result {
        Ok(version) => return Json(json!({"id": ticket_id, "version": version})).into_response(),
        Err(PersistReviewError::TicketNotFound) => return ticket_not_found(),
        Err(PersistReviewError::ManagerRequired) => (
            StatusCode::FORBIDDEN, "manager_required", "Only an active IT manager can review tickets.",
        ),
        Err(Review(ReviewError::VersionConflict)) => (
            StatusCode::CONFLICT, "version_conflict", "Ticket version has changed.",
        ),
        Err(Review(ReviewError::NotPendingReview)) => (
            StatusCode::CONFLICT, "not_pending_review", "Ticket is no longer pending review.",
        ),
        Err(Review(ReviewError::Assignment(AssignmentError::AssigneeRequired))) => (
            StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "Select a team member for this ticket.",
        ),
        Err(Review(ReviewError::Assignment(AssignmentError::P1RequiresReviewingManager))) => (
            StatusCode::CONFLICT, "ineligible_assignee", "The reviewing manager must handle a P1 ticket.",
        ),
        Err(Review(ReviewError::Assignment(AssignmentError::IneligibleMember))) => (
            StatusCode::CONFLICT, "ineligible_assignee", "Select an active team member for this category.",
        ),
        Err(PersistReviewError::Database(_)) => {
            tracing::error!("Ticket review failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Could not review ticket.")
        }
    };
    (status, Json(json!({"error": {"code": code, "message": message}}))).into_response()
}
