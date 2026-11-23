use super::{
    dto::{CreateTicketInput, ListTicketsQuery, ReviewTicketInput, TicketStatusInput},
    routes,
};
use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use sqlx::PgPool;
use uuid::Uuid;

/// Login middleware must insert this extension only after verifying credentials.
/// Request headers and JSON fields do not establish an identity.
#[derive(Clone, Copy)]
pub struct AuthenticatedActor {
    pub user_id: Uuid,
}

impl FromRequestParts<PgPool> for AuthenticatedActor {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, pool: &PgPool) -> Result<Self, Response> {
        let actor = parts
            .extensions
            .get::<Self>()
            .copied()
            .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "authentication_required"))?;
        let active =
            sqlx::query_scalar::<_, bool>("SELECT account_status='active' FROM users WHERE id=$1")
                .bind(actor.user_id)
                .fetch_optional(pool)
                .await;
        match active {
            Ok(Some(true)) => Ok(actor),
            Ok(_) => Err(error(StatusCode::UNAUTHORIZED, "authentication_required")),
            Err(_) => {
                tracing::error!("Ticket actor lookup failed");
                Err(error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error"))
            }
        }
    }
}

fn error(status: StatusCode, code: &str) -> Response {
    (
        status,
        Json(serde_json::json!({"error": {"code": code, "message": status.canonical_reason()}})),
    )
        .into_response()
}

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/api/v1/tickets", post(create).get(list))
        .route("/api/v1/tickets/{id}", get(detail))
        .route("/api/v1/tickets/{id}/events", get(history))
        .route("/api/v1/tickets/{id}/review", post(review))
        .route("/api/v1/tickets/{id}/status", axum::routing::patch(status))
}

async fn create(
    actor: AuthenticatedActor,
    State(pool): State<PgPool>,
    body: Result<Json<CreateTicketInput<Uuid>>, axum::extract::rejection::JsonRejection>,
) -> Response {
    match body {
        Ok(Json(input)) => routes::create_ticket_response(&pool, actor.user_id, &input).await,
        Err(rejection) => error(rejection.status(), "invalid_json"),
    }
}

async fn list(
    actor: AuthenticatedActor,
    State(pool): State<PgPool>,
    query: Result<Query<ListTicketsQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    routes::list_tickets_response(&pool, actor.user_id, query).await
}

async fn detail(
    actor: AuthenticatedActor,
    State(pool): State<PgPool>,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
) -> Response {
    match path {
        Ok(Path(id)) => routes::get_ticket_response(&pool, actor.user_id, id).await,
        Err(_) => error(StatusCode::BAD_REQUEST, "invalid_id"),
    }
}

async fn history(
    actor: AuthenticatedActor,
    State(pool): State<PgPool>,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
) -> Response {
    match path {
        Ok(Path(id)) => routes::get_ticket_events_response(&pool, actor.user_id, id).await,
        Err(_) => error(StatusCode::BAD_REQUEST, "invalid_id"),
    }
}

async fn review(
    actor: AuthenticatedActor,
    State(pool): State<PgPool>,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
    body: Result<Json<ReviewTicketInput>, axum::extract::rejection::JsonRejection>,
) -> Response {
    match (path, body) {
        (Ok(Path(id)), Ok(Json(input))) => {
            routes::review_ticket_response(&pool, actor.user_id, id, input).await
        }
        (Err(_), _) => error(StatusCode::BAD_REQUEST, "invalid_id"),
        (_, Err(rejection)) => error(rejection.status(), "invalid_json"),
    }
}

async fn status(
    actor: AuthenticatedActor,
    State(pool): State<PgPool>,
    path: Result<Path<Uuid>, axum::extract::rejection::PathRejection>,
    body: Result<Json<TicketStatusInput>, axum::extract::rejection::JsonRejection>,
) -> Response {
    match (path, body) {
        (Ok(Path(id)), Ok(Json(input))) => {
            routes::ticket_status_response(&pool, actor.user_id, id, input).await
        }
        (Err(_), _) => error(StatusCode::BAD_REQUEST, "invalid_id"),
        (_, Err(rejection)) => error(rejection.status(), "invalid_json"),
    }
}
