mod api;
mod config;
mod error;
mod kafka;
mod repositories;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::handlers::AppState, config::Config, kafka::KafkaConsumerService,
    repositories::SettingsRepository,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,heatpump_settings_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting heatpump-settings-api");

    // Load configuration
    let config = Config::load()?;

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(config.database_url())
        .await?;
    tracing::info!("Database connection established");

    // Create repository
    let repository = Arc::new(SettingsRepository::new(db_pool.clone()));

    // Create Kafka consumer service
    tracing::info!("Initializing Kafka consumer...");
    let kafka_consumer =
        KafkaConsumerService::new(&config.kafka, SettingsRepository::new(db_pool))?;

    // Spawn Kafka consumer task
    let kafka_handle = tokio::spawn(async move {
        kafka_consumer.run().await;
    });

    // Create API server
    let app_state = AppState {
        repository: repository.clone(),
    };
    let app = api::create_router(app_state);

    let bind_addr = config.api_bind_address();
    tracing::info!("Starting API server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Wait for Kafka consumer to finish (it won't unless shutdown)
    kafka_handle.abort();

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
