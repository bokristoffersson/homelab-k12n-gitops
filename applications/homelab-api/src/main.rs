mod api;
mod auth;
mod config;
mod db;
mod error;
mod repositories;

use config::Config;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    info!("Starting homelab-api");

    let cfg_path = std::env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.yaml".into());
    let cfg = Config::load(&cfg_path)?;
    info!("Configuration loaded");

    let pool = db::connect(&cfg.database.url).await?;
    sqlx::query("SELECT 1").execute(&pool).await?;
    info!("Connected to database");

    let router = api::create_router(pool.clone(), cfg.clone());
    let addr = format!("{}:{}", cfg.api.host, cfg.api.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    info!("API server listening on {}", addr);

    // Set up graceful shutdown
    let serve = axum::serve(listener, router);
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received");
    };

    if let Err(e) = serve.with_graceful_shutdown(shutdown).await {
        tracing::error!(error = %e, "API server error");
    }

    info!("Application shutdown complete");
    Ok(())
}
