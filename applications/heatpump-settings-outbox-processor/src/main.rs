mod config;
mod db;
mod outbox;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting heatpump-settings-outbox-processor");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");
    info!("MQTT broker: {}", config.mqtt_broker);
    info!("Poll interval: {}s", config.poll_interval_secs);

    // Connect to database
    let pool = db::connect(&config.database_url).await?;
    info!("Connected to database successfully");

    // Connect to MQTT broker
    // Parse broker address (format: "host:port")
    let parts: Vec<&str> = config.mqtt_broker.split(':').collect();
    let host = parts
        .get(0)
        .unwrap_or(&"mosquitto.mosquitto.svc.cluster.local")
        .to_string();
    let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1883);

    let mut mqtt_options = MqttOptions::new("heatpump-settings-outbox-processor", host, port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    info!("MQTT client created");

    // Spawn MQTT event loop handler
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    error!("MQTT event loop error: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    // Main processing loop
    loop {
        match process_pending_entries(&pool, &mqtt_client).await {
            Ok(processed) => {
                if processed > 0 {
                    info!("Processed {} pending outbox entries", processed);
                }
            }
            Err(e) => {
                error!("Error processing outbox entries: {}", e);
            }
        }

        sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

async fn process_pending_entries(
    pool: &sqlx::PgPool,
    mqtt_client: &AsyncClient,
) -> Result<usize, Box<dyn std::error::Error>> {
    let entries = outbox::get_pending_entries(pool, 10).await?;

    if entries.is_empty() {
        return Ok(0);
    }

    info!("Found {} pending entries to process", entries.len());

    for entry in &entries {
        // Build MQTT topic: heatpump/{device_id}/command
        let topic = format!("heatpump/{}/command", entry.aggregate_id);
        let payload = entry.payload.to_string();

        info!(
            "Publishing outbox entry {} to topic '{}': {}",
            entry.id, topic, payload
        );

        // Publish to MQTT
        match mqtt_client
            .publish(&topic, QoS::AtLeastOnce, false, payload.as_bytes())
            .await
        {
            Ok(_) => {
                // Mark as published
                match outbox::mark_published(pool, entry.id).await {
                    Ok(_) => {
                        info!("✓ Published outbox entry {}", entry.id);
                    }
                    Err(e) => {
                        error!("Failed to mark entry {} as published: {}", entry.id, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to publish entry {}: {}", entry.id, e);

                // Check if max retries exceeded
                if entry.retry_count + 1 >= entry.max_retries {
                    warn!(
                        "Entry {} exceeded max retries ({}), marking as failed",
                        entry.id, entry.max_retries
                    );
                    match outbox::mark_failed(pool, entry.id, &e.to_string()).await {
                        Ok(_) => {
                            error!("✗ Marked outbox entry {} as failed", entry.id);
                        }
                        Err(db_err) => {
                            error!("Failed to mark entry {} as failed: {}", entry.id, db_err);
                        }
                    }
                } else {
                    // Increment retry count
                    match outbox::increment_retry(pool, entry.id, &e.to_string()).await {
                        Ok(_) => {
                            warn!(
                                "↻ Incremented retry count for entry {} ({}/{})",
                                entry.id,
                                entry.retry_count + 1,
                                entry.max_retries
                            );
                        }
                        Err(db_err) => {
                            error!(
                                "Failed to increment retry for entry {}: {}",
                                entry.id, db_err
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(entries.len())
}
