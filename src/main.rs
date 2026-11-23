mod config;
mod db;
mod telemetry;

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

    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .map_err(|_| "Could not query PostgreSQL")?;

    tracing::info!(
        database = %database_name,
        "PostgreSQL connection established"
    );

    let app = assetdesk::build_app(pool);
    let listener = tokio::net::TcpListener::bind(config.address).await?;

    tracing::info!(address = %config.address, "HTTP server listening");
    axum::serve(listener, app).await?;

    Ok(())
}
