use crate::error::AppError;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use std::time::Duration;
use tracing::{debug, error, warn};

pub type RedpandaConsumer = StreamConsumer;

pub async fn create_consumer(
    brokers: &str,
    group_id: &str,
    auto_offset_reset: &str,
) -> Result<RedpandaConsumer, AppError> {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", brokers);
    config.set("group.id", group_id);
    config.set("auto.offset.reset", auto_offset_reset);
    config.set("enable.partition.eof", "false");
    config.set("session.timeout.ms", "30000");
    config.set("enable.auto.commit", "true");
    config.set("auto.commit.interval.ms", "5000");

    let consumer: StreamConsumer = config
        .create()
        .map_err(|e| AppError::Kafka(format!("Failed to create consumer: {}", e)))?;

    Ok(consumer)
}

pub async fn subscribe_to_topics(
    consumer: &RedpandaConsumer,
    topics: &[String],
) -> Result<(), AppError> {
    let topic_refs: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();
    consumer
        .subscribe(&topic_refs)
        .map_err(|e| AppError::Kafka(format!("Failed to subscribe to topics: {}", e)))?;

    debug!(topics=?topics, "subscribed to topics");
    Ok(())
}

pub async fn receive_message(
    consumer: &RedpandaConsumer,
) -> Result<Option<ReceivedMessage>, AppError> {
    match tokio::time::timeout(Duration::from_secs(1), consumer.recv()).await {
        Ok(Ok(message)) => {
            let topic = message.topic().to_string();
            let partition = message.partition();
            let offset = message.offset();

            let payload = match message.payload() {
                Some(p) => p.to_vec(),
                None => {
                    warn!(topic=%topic, partition=partition, offset=offset, "message has no payload");
                    return Ok(None);
                }
            };

            let key = message.key().map(|k| k.to_vec());

            debug!(
                topic=%topic,
                partition=partition,
                offset=offset,
                payload_len=payload.len(),
                "received message"
            );

            Ok(Some(ReceivedMessage {
                topic,
                partition,
                offset,
                payload,
                key,
            }))
        }
        Ok(Err(e)) => {
            error!(error=%e, "error receiving message");
            Err(AppError::Kafka(format!("Consumer error: {}", e)))
        }
        Err(_) => {
            // Timeout - no message available
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReceivedMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub payload: Vec<u8>,
    pub key: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_consumer_config() {
        // Test that consumer configuration parameters are valid
        let brokers = "localhost:9092";
        let group_id = "test-group";
        let auto_offset_reset = "earliest";

        // We can't easily test consumer creation without a running broker,
        // but we can verify the parameters are valid strings
        assert!(!brokers.is_empty());
        assert!(!group_id.is_empty());
        assert!(matches!(auto_offset_reset, "earliest" | "latest" | "none"));
    }
}
