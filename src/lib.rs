pub mod features;

use axum::{Json, Router, body::Body, http::Request, routing::get};
use axum::{extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::PgPool;
use std::time::Duration;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn ready(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let query = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.pool);

    match tokio::time::timeout(Duration::from_secs(2), query).await {
        Ok(Ok(1)) => (StatusCode::OK, Json(HealthResponse { status: "ready" })),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "not_ready",
            }),
        ),
    }
}

pub fn build_app(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .nest("/api/v1/users", features::users::router())
        .with_state(AppState { pool })
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    path = %request.uri().path(),
                )
            }),
        )
}
