use crate::error::Result;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub type EnergyMessage = Value;

/// Create a Kafka consumer configured for the energy topic
pub fn create_consumer(
    brokers: &str,
    group_id: &str,
    auto_offset_reset: &str,
) -> Result<StreamConsumer> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", auto_offset_reset)
        .set("session.timeout.ms", "6000")
        .set("enable.partition.eof", "false")
        .create()?;

    Ok(consumer)
}

/// Run the Kafka consumer loop, broadcasting messages to all WebSocket clients
pub async fn run_consumer(
    consumer: StreamConsumer,
    topic: String,
    tx: broadcast::Sender<EnergyMessage>,
) -> Result<()> {
    // Subscribe to the topic
    consumer.subscribe(&[&topic])?;
    info!("Subscribed to Kafka topic: {}", topic);

    loop {
        match consumer.recv().await {
            Ok(message) => {
                // Extract payload
                let payload = match message.payload() {
                    Some(p) => p,
                    None => {
                        warn!("Received message with no payload");
                        continue;
                    }
                };

                // Parse JSON
                match serde_json::from_slice::<EnergyMessage>(payload) {
                    Ok(msg) => {
                        // Broadcast to all connected clients
                        let receiver_count = tx.receiver_count();
                        if receiver_count > 0 {
                            match tx.send(msg.clone()) {
                                Ok(count) => {
                                    tracing::debug!(
                                        "Broadcast energy message to {} clients",
                                        count
                                    );
                                }
                                Err(e) => {
                                    warn!("Failed to broadcast message: {}", e);
                                }
                            }
                        } else {
                            tracing::debug!("No WebSocket clients connected, skipping broadcast");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse message as JSON: {}", e);
                        // Log the raw payload for debugging
                        if let Ok(s) = std::str::from_utf8(payload) {
                            tracing::debug!("Raw payload: {}", s);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Kafka error: {}", e);
                // Sleep briefly before retrying
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_consumer() {
        // This test will fail if Kafka is not available, which is expected
        // In CI, this will be tested in the e2e tests with Docker
        let result = create_consumer("localhost:9092", "test-group", "latest");
        // Just verify it creates a consumer (even if it can't connect)
        assert!(result.is_ok());
    }
}
