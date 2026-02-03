use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::util::Timeout;
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::outbox;

/// Start Kafka consumer to listen for telemetry confirmations
pub async fn start_confirmation_listener(
    pool: PgPool,
    brokers: String,
    topic: String,
    group_id: String,
) {
    info!(
        "Starting Kafka confirmation listener on topic '{}' with group '{}'",
        topic, group_id
    );

    let consumer: StreamConsumer = match ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("group.id", &group_id)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "latest") // Only process new telemetry
        .set("session.timeout.ms", "30000")
        .set("enable.partition.eof", "false")
        .create()
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create Kafka consumer: {}", e);
            return;
        }
    };

    if let Err(e) = consumer.subscribe(&[&topic]) {
        error!("Failed to subscribe to topic '{}': {}", topic, e);
        return;
    }

    info!("Successfully subscribed to Kafka topic '{}'", topic);

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    match std::str::from_utf8(payload) {
                        Ok(payload_str) => {
                            if let Err(e) = process_telemetry_message(&pool, payload_str).await {
                                error!("Error processing telemetry message: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse message payload as UTF-8: {}", e);
                        }
                    }
                } else {
                    warn!("Received message with no payload");
                }
            }
            Err(e) => {
                error!("Kafka consumer error: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

/// Process a telemetry message and mark corresponding outbox entries as confirmed
async fn process_telemetry_message(
    pool: &PgPool,
    payload: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse JSON payload
    let message: Value = serde_json::from_str(payload)?;

    // Extract device_id from tags.device_id
    let device_id = message
        .get("tags")
        .and_then(|tags| tags.get("device_id"))
        .and_then(|id| id.as_str())
        .ok_or("Missing or invalid tags.device_id")?;

    info!("Received telemetry for device: {}", device_id);

    // Mark outbox entries for this device as confirmed
    match outbox::mark_confirmed(pool, device_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                info!(
                    "✓ Confirmed {} outbox entry/entries for device {}",
                    rows_affected, device_id
                );
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to mark outbox entries as confirmed: {}", e);
            Err(Box::new(e))
        }
    }
}
