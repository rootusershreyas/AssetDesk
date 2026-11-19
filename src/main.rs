mod config;
mod db;
mod features;
mod telemetry;

use axum::routing::post;
use axum::{Json, Router, body::Body, http::Request, routing::get};
use axum::{extract::State, http::StatusCode};
use features::categories::routes::{
    create_category, get_category, list_categories, update_category,
};
use serde::Serialize;
use sqlx::PgPool;
use std::time::Duration;
use tower_http::trace::TraceLayer;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
async fn ready(State(pool): State<PgPool>) -> (StatusCode, Json<HealthResponse>) {
    let query = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&pool);

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenvy::from_path(".env") {
        Ok(()) => {}
        Err(error) if error.not_found() => {}
        Err(_) => {
            return Err("Cannot load .env; check its format and permissions".into());
        }
    }

    telemetry::init()?;

    let config = config::Config::from_env()?;
    let pool = db::connect(&config.database_url, config.database_max_connections)
        .await
        .map_err(|_| "Could not connect to PostgreSQL; check database configuration and health")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|_| "Could not apply PostgreSQL migrations")?;

    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .map_err(|_| "Could not query PostgreSQL")?;

    tracing::info!(
        database = %database_name,
        "PostgreSQL connection established"
    );

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route(
            "/api/v1/categories",
            post(create_category).get(list_categories),
        )
        .route(
            "/api/v1/categories/{id}",
            get(get_category).patch(update_category),
        )
        .with_state(pool)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    path = request.uri().path(),
                )
            }),
        );

    let listener = tokio::net::TcpListener::bind(config.address).await?;

    tracing::info!(
        address = %config.address,
        "HTTP server listening"
    );

    axum::serve(listener, app).await?;

    Ok(())
}
