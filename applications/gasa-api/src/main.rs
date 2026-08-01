mod auth;
mod config;
mod email;
mod error;
mod handlers;
mod ics;
mod models;
mod repository;
mod routes;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::Config, email::EmailService, handlers::AppState, repository::SlotsRepository};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,gasa_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting gasa-api");

    let config = Config::load()?;

    tracing::info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(config.database_url())
        .await?;
    tracing::info!("Database connection established");

    let email = EmailService::new(config.smtp.clone());

    let bind_addr = config.api_bind_address();
    let app_state = AppState {
        repository: SlotsRepository::new(db_pool),
        config: Arc::new(config),
        email,
    };
    let app = routes::create_router(app_state);

    tracing::info!("Starting API server on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Application shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received");
}
