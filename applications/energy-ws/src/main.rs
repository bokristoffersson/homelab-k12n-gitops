use axum::{routing::get, Router};
use energy_ws::{
    config::Config,
    kafka::{create_consumer, run_consumer},
    ws::{health_check, ws_handler, AppState},
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "energy_ws=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting energy-ws service");

    // Load configuration
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());

    let config = Config::load(&config_path)?;
    info!("Configuration loaded from: {}", config_path);

    // Create broadcast channel for Kafka messages
    let (broadcast_tx, _broadcast_rx) = broadcast::channel(100);

    // Create Kafka consumer
    let consumer = create_consumer(
        &config.kafka.brokers,
        &config.kafka.group_id,
        &config.kafka.auto_offset_reset,
    )?;
    info!(
        "Kafka consumer created: group_id={}, brokers={}",
        config.kafka.group_id, config.kafka.brokers
    );

    // Spawn Kafka consumer task
    let topic = config.kafka.topic.clone();
    let tx = broadcast_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = run_consumer(consumer, topic, tx).await {
            error!("Kafka consumer error: {}", e);
        }
    });

    // Create application state
    let state = Arc::new(AppState::new(
        config.auth.jwt_secret.clone().unwrap_or_default(),
        broadcast_tx,
        config.server.max_connections,
    ));

    // Build Axum application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws/energy", get(ws_handler))
        .with_state(state);

    // Start HTTP server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server ready to accept WebSocket connections");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully");
        },
    }
}
