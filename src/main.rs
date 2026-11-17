mod config;
mod telemetry;

use axum::{Json, Router, body::Body, http::Request, routing::get};
use serde::Serialize;
use tower_http::trace::TraceLayer;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
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

    let app = Router::new().route("/health", get(health)).layer(
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
