use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::outbox;

/// Start Kafka consumer to listen for telemetry confirmations
pub async fn start_confirmation_listener(
    pool: PgPool,
    brokers: String,
    heatpump_topic: String,
    plug_topic: String,
    group_id: String,
) {
    info!(
        "Starting Kafka confirmation listener on topics '{}', '{}' with group '{}'",
        heatpump_topic, plug_topic, group_id
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

    if let Err(e) = consumer.subscribe(&[&heatpump_topic, &plug_topic]) {
        error!(
            "Failed to subscribe to topics '{}', '{}': {}",
            heatpump_topic, plug_topic, e
        );
        return;
    }

    info!(
        "Successfully subscribed to Kafka topics '{}', '{}'",
        heatpump_topic, plug_topic
    );

    loop {
        match consumer.recv().await {
            Ok(message) => {
                let topic = message.topic().to_string();
                if let Some(payload) = message.payload() {
                    match std::str::from_utf8(payload) {
                        Ok(payload_str) => {
                            let result = if topic == plug_topic {
                                process_plug_telemetry_message(&pool, payload_str).await
                            } else {
                                process_telemetry_message(&pool, payload_str).await
                            };
                            if let Err(e) = result {
                                error!(
                                    "Error processing telemetry message from '{}': {}",
                                    topic, e
                                );
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

/// Process a heatpump telemetry message and mark corresponding outbox entries as confirmed
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

/// Process a plug telemetry message: the device echoing a power state is the
/// proof that a command took effect, so only commands whose action matches the
/// echoed state are confirmed.
async fn process_plug_telemetry_message(
    pool: &PgPool,
    payload: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (plug_id, action) =
        parse_plug_state(payload).ok_or("Missing plug_id or status in plug telemetry")?;

    match outbox::mark_plug_confirmed(pool, &plug_id, action).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                info!(
                    "✓ Confirmed {} plug command(s) for {} (device state: {})",
                    rows_affected, plug_id, action
                );
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to mark plug commands as confirmed: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Extract (plug_id, "ON"/"OFF") from a homelab-plug-telemetry message
fn parse_plug_state(payload: &str) -> Option<(String, &'static str)> {
    let message: Value = serde_json::from_str(payload).ok()?;
    let plug_id = message.get("plug_id")?.as_str()?.to_string();
    let status = message.get("status")?.as_bool()?;
    Some((plug_id, if status { "ON" } else { "OFF" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plug_state_on() {
        let payload = r#"{"ts":"2026-09-07T10:00:00Z","plug_id":"tasmota_living_room","status":true,"wifi_rssi":-55,"uptime_seconds":1234}"#;
        assert_eq!(
            parse_plug_state(payload),
            Some(("tasmota_living_room".to_string(), "ON"))
        );
    }

    #[test]
    fn test_parse_plug_state_off() {
        // stat/+/POWER-derived messages carry only plug_id and status
        let payload = r#"{"ts":"2026-09-07T10:00:00Z","plug_id":"tasmota_bedroom","status":false}"#;
        assert_eq!(
            parse_plug_state(payload),
            Some(("tasmota_bedroom".to_string(), "OFF"))
        );
    }

    #[test]
    fn test_parse_plug_state_missing_fields() {
        assert_eq!(parse_plug_state(r#"{"plug_id":"tasmota_x"}"#), None);
        assert_eq!(parse_plug_state(r#"{"status":true}"#), None);
        assert_eq!(parse_plug_state("not json"), None);
        // status must be a boolean, not a string
        assert_eq!(
            parse_plug_state(r#"{"plug_id":"tasmota_x","status":"ON"}"#),
            None
        );
    }
}
