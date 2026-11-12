use assetdesk::{app, config::Config, db, telemetry};

#[tokio::main]
async fn main() -> Result<(), String> {
    dotenvy::dotenv().ok();
    telemetry::init();
    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url, config.database_max_connections)
        .await
        .map_err(|_| "Could not connect to PostgreSQL; check DATABASE_URL and database health.")?;
    if db::migrate(&pool).await.is_err() {
        pool.close().await;
        return Err("Could not apply database migrations; inspect the migration setup.".into());
    }
    let listener = tokio::net::TcpListener::bind(config.address)
        .await
        .map_err(|_| "Could not bind the HTTP address; check HOST and PORT.")?;
    tracing::info!(address = %config.address, "AssetDesk listening; database migrations applied");
    let result = axum::serve(listener, app::router(pool.clone()))
        .with_graceful_shutdown(shutdown_signal())
        .await;
    pool.close().await;
    result.map_err(|_| "HTTP server stopped unexpectedly.".into())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        } else {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("Shutting down HTTP server");
}
