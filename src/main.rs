mod config;

use axum::{Json, Router, routing::get};
use serde::Serialize;

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

    let config = config::Config::from_env()?;

    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(config.address).await?;

    println!("Listening on http://{}", config.address);

    axum::serve(listener, app).await?;

    Ok(())
}
