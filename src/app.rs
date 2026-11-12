use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use utoipa::ToSchema;

use crate::{error::AppError, features::assets, openapi, state::AppState};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api-docs/openapi.json", get(specification))
        .nest("/api/v1", assets::router())
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { pool })
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Operations",
    responses((status = 200, description = "Process is alive", body = HealthResponse))
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "Operations",
    responses(
        (status = 200, description = "PostgreSQL is reachable", body = HealthResponse),
        (status = 503, description = "Database unavailable", body = crate::error::ErrorResponse)
    )
)]
pub async fn ready(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    match tokio::time::timeout(
        Duration::from_secs(2),
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.pool),
    )
    .await
    {
        Ok(Ok(1)) => Ok(Json(HealthResponse { status: "ready" })),
        _ => Err(AppError::Unavailable),
    }
}

async fn specification() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi::document())
}

async fn not_found() -> AppError {
    AppError::NotFound("Route")
}

async fn method_not_allowed() -> impl axum::response::IntoResponse {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        Json(crate::error::ErrorResponse {
            error: crate::error::ErrorDetails {
                code: "method_not_allowed".into(),
                message: "This HTTP method is not supported for this route.".into(),
            },
        }),
    )
}
