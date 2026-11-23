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

/// The authenticated actor is supplied separately from the requested status.
pub async fn ticket_status_response(
    pool: &PgPool,
    actor: Uuid,
    ticket_id: Uuid,
    input: super::dto::TicketStatusInput,
) -> Response {
    use super::{
        dto::TicketStatusInput,
        lifecycle::{PersistStartError, start_assigned_ticket},
        model::StartWorkError,
    };

    let expected_version = match input {
        TicketStatusInput::Resolved { expected_version } => {
            return confirm_resolution_response(pool, actor, ticket_id, expected_version).await;
        }
        TicketStatusInput::InProgress { expected_version } => expected_version,
        TicketStatusInput::AwaitingEmployeeConfirmation { expected_version, resolution } => {
            return submit_resolution_response(pool, actor, ticket_id, expected_version, &resolution).await;
        }
    };
    let (status, code, message) = if expected_version < 1 {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "Expected version must be positive.",
        )
    } else {
        match start_assigned_ticket(pool, ticket_id, actor, expected_version).await {
            Ok(version) => {
                return Json(json!({"id": ticket_id, "status": "in_progress", "version": version}))
                    .into_response();
            }
            Err(PersistStartError::TicketNotFound) => return ticket_not_found(),
            Err(PersistStartError::Rule(StartWorkError::NotAssignedHandler)) => (
                StatusCode::FORBIDDEN,
                "not_assigned_handler",
                "Only the assigned handler can start work.",
            ),
            Err(PersistStartError::Rule(StartWorkError::VersionConflict)) => (
                StatusCode::CONFLICT,
                "version_conflict",
                "Ticket version has changed.",
            ),
            Err(PersistStartError::Rule(
                StartWorkError::InvalidStatus | StartWorkError::IncompleteReview,
            )) => (
                StatusCode::CONFLICT,
                "invalid_transition",
                "Only a reviewed open ticket can be started.",
            ),
            Err(PersistStartError::IneligibleHandler) => (
                StatusCode::CONFLICT,
                "ineligible_handler",
                "The assigned handler is no longer eligible.",
            ),
            Err(PersistStartError::Database(_)) => {
                tracing::error!("Ticket start failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Could not start ticket.",
                )
            }
        }
    };
    (
        status,
        Json(json!({"error": {"code": code, "message": message}})),
    )
        .into_response()
}

async fn submit_resolution_response(
    pool: &PgPool,
    actor: Uuid,
    ticket_id: Uuid,
    expected_version: i64,
    resolution: &str,
) -> Response {
    use super::{
        lifecycle::{PersistResolutionError, resolve_assigned_ticket},
        service::SubmitResolutionError,
    };
    let (status, code, message) = if expected_version < 1 {
        (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "Expected version must be positive.")
    } else {
        match resolve_assigned_ticket(pool, ticket_id, actor, expected_version, resolution).await {
            Ok(version) => return Json(json!({
                "id": ticket_id, "status": "awaiting_employee_confirmation", "version": version
            })).into_response(),
            Err(PersistResolutionError::TicketNotFound) => return ticket_not_found(),
            Err(PersistResolutionError::Rule(SubmitResolutionError::NotAssignedHandler)) => (
                StatusCode::FORBIDDEN, "not_assigned_handler", "Only the assigned handler can submit a resolution.",
            ),
            Err(PersistResolutionError::Rule(SubmitResolutionError::VersionConflict)) => (
                StatusCode::CONFLICT, "version_conflict", "Ticket version has changed.",
            ),
            Err(PersistResolutionError::Rule(SubmitResolutionError::InvalidText(_))) => (
                StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "Resolution must contain 1 to 8000 characters.",
            ),
            Err(PersistResolutionError::Rule(SubmitResolutionError::InvalidStatus | SubmitResolutionError::IncompleteReview)) => (
                StatusCode::CONFLICT, "invalid_transition", "Only reviewed work in progress can receive a resolution.",
            ),
            Err(PersistResolutionError::IneligibleHandler) => (
                StatusCode::CONFLICT, "ineligible_handler", "The assigned handler is no longer eligible.",
            ),
            Err(PersistResolutionError::Database(_)) => {
                tracing::error!("Resolution submission failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Could not submit resolution.")
            }
        }
    };
    (status, Json(json!({"error": {"code": code, "message": message}}))).into_response()
}

async fn confirm_resolution_response(
    pool: &PgPool,
    actor: Uuid,
    ticket_id: Uuid,
    expected_version: i64,
) -> Response {
    use super::{
        lifecycle::{PersistConfirmationError, confirm_ticket_resolution},
        service::ConfirmResolutionError,
    };
    let (status, code, message) = if expected_version < 1 {
        (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", "Expected version must be positive.")
    } else {
        match confirm_ticket_resolution(pool, ticket_id, actor, expected_version).await {
            Ok(version) => return Json(json!({
                "id": ticket_id, "status": "resolved", "version": version
            })).into_response(),
            Err(PersistConfirmationError::TicketNotFound) => return ticket_not_found(),
            Err(PersistConfirmationError::Rule(ConfirmResolutionError::NotRequester)) => (
                StatusCode::FORBIDDEN, "not_requester", "Only the requester can confirm the resolution.",
            ),
            Err(PersistConfirmationError::Rule(ConfirmResolutionError::VersionConflict)) => (
                StatusCode::CONFLICT, "version_conflict", "Ticket version has changed.",
            ),
            Err(PersistConfirmationError::Rule(ConfirmResolutionError::InvalidStatus)) => (
                StatusCode::CONFLICT, "invalid_transition", "Ticket must be awaiting employee confirmation.",
            ),
            Err(PersistConfirmationError::Database(_)) => {
                tracing::error!("Resolution confirmation failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Could not confirm resolution.")
            }
        }
    };
    (status, Json(json!({"error": {"code": code, "message": message}}))).into_response()
}
