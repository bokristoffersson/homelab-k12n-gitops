mod config;
mod db;
mod error;
mod ingest;
mod mapping;
mod redpanda;

use config::Config;
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
        pool,
        cfg.database.write.batch_size,
        cfg.database.write.linger_ms,
    );

    let sig = tokio::signal::ctrl_c();
    tokio::pin!(sig);
    loop {
        tokio::select! {
            biased;
            _ = &mut sig => {
                info!("shutdown requested");
                break;
            }
            res = redpanda::receive_message(&consumer) => {
                match res {
                    Ok(Some(msg)) => {
                        if let Err(e) = ingestor.handle_message(&cfg.pipelines, &msg.topic, &msg.payload).await {
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
        }
    }

    Ok(())
}
