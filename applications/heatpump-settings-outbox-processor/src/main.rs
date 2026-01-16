mod config;
mod db;

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting heatpump-settings-outbox-processor");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Connect to database
    match db::connect(&config.database_url).await {
        Ok(_pool) => {
            info!("Connected to database successfully");

            // TODO: Poll outbox table
            // TODO: Publish to MQTT
            // TODO: Listen for Kafka confirmations

            loop {
                info!("Polling outbox table...");
                // TODO: Implement polling logic
                sleep(Duration::from_secs(config.poll_interval_secs)).await;
            }
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(e.into());
        }
    }
}
