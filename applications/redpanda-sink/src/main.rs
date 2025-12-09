mod api;
mod auth;
mod config;
mod db;
mod error;
mod ingest;
mod mapping;
mod redpanda;
mod repositories;

use config::Config;
use tokio::sync::broadcast;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    let cfg_path =
        std::env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.example.yaml".into());
    let mut cfg = Config::load(&cfg_path)?;
    if let Ok(url) = std::env::var("DATABASE_URL") {
        cfg.database.url = url;
    }
    if let Ok(brokers) = std::env::var("REDPANDA_BROKERS") {
        cfg.redpanda.brokers = brokers;
    }
    info!("loaded config; pipelines: {}", cfg.pipelines.len());

    let pool = db::connect(&cfg.database.url).await?;
    sqlx::query("SELECT 1").execute(&pool).await?;
    info!("connected to database");

    let consumer = redpanda::create_consumer(
        &cfg.redpanda.brokers,
        &cfg.redpanda.group_id,
        &cfg.redpanda.auto_offset_reset,
    )
    .await?;
    info!(
        brokers = %cfg.redpanda.brokers,
        group_id = %cfg.redpanda.group_id,
        "connected to Redpanda"
    );

    // Subscribe to all topics from pipelines
    let topics: Vec<String> = cfg.pipelines.iter().map(|p| p.topic.clone()).collect();
    redpanda::subscribe_to_topics(&consumer, &topics).await?;
    info!("subscribed to {} topic(s)", topics.len());

    let ingestor = ingest::Ingestor::new(
        pool.clone(),
        cfg.database.write.batch_size,
        cfg.database.write.linger_ms,
    );

    // Create shutdown signal channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let shutdown_tx_for_api = shutdown_tx.clone();

    // Start API server if enabled
    let api_handle = if let Some(api_cfg) = &cfg.api {
        if api_cfg.enabled {
            info!(
                host = %api_cfg.host,
                port = api_cfg.port,
                "starting API server"
            );

            let router = api::create_router(pool.clone(), cfg.clone());
            let addr = format!("{}:{}", api_cfg.host, api_cfg.port);

            let mut shutdown_rx_api = shutdown_tx_for_api.subscribe();
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(|e| anyhow::anyhow!("failed to bind to {}: {}", addr, e))?;

            info!("API server listening on {}", addr);

            Some(tokio::spawn(async move {
                let serve = axum::serve(listener, router);
                let shutdown = async move {
                    shutdown_rx_api.recv().await.ok();
                    info!("API server shutdown signal received");
                };

                if let Err(e) = serve.with_graceful_shutdown(shutdown).await {
                    warn!(error = %e, "API server error");
                }
            }))
        } else {
            info!("API server disabled in config");
            None
        }
    } else {
        info!("API config not provided, API server disabled");
        None
    };

    // Main consumer loop with graceful shutdown
    let pipelines = cfg.pipelines.clone();
    let consumer_handle = tokio::spawn(async move {
        loop {
            match redpanda::receive_message(&consumer).await {
                Ok(Some(msg)) => {
                    if let Err(e) = ingestor
                        .handle_message(&pipelines, &msg.topic, &msg.payload)
                        .await
                    {
                        warn!(topic=%msg.topic, error=%e, "processing failed for incoming message");
                    }
                }
                Ok(None) => continue,
                Err(e) => {
                    warn!("redpanda error: {e}; continuing after short delay");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    });

    // Wait for shutdown signal or either task to complete
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("shutdown requested");
            let _ = shutdown_tx.send(());
        }
        _ = consumer_handle => {
            info!("consumer task completed");
            let _ = shutdown_tx.send(());
        }
        result = async {
            if let Some(handle) = api_handle {
                handle.await
            } else {
                Ok(())
            }
        } => {
            if let Err(e) = result {
                warn!(error = %e, "API server task error");
            }
            info!("API server task completed");
            let _ = shutdown_tx.send(());
        }
    }

    // Give a moment for graceful shutdown
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    info!("application shutdown complete");

    Ok(())
}
