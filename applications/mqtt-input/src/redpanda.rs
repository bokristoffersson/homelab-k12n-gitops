use crate::error::AppError;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;
use tracing::{debug, error};

pub type RedpandaProducer = FutureProducer;

pub async fn create_producer(brokers: &str) -> Result<RedpandaProducer, AppError> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("queue.buffering.max.messages", "100000")
        .set("queue.buffering.max.kbytes", "1048576")
        .set("batch.num.messages", "10000")
        .create()
        .map_err(|e| AppError::Kafka(format!("Failed to create producer: {}", e)))?;

    Ok(producer)
}

pub async fn publish_message(
    producer: &RedpandaProducer,
    topic: &str,
    key: Option<&str>,
    payload: &[u8],
) -> Result<(), AppError> {
    let mut record = FutureRecord::to(topic).payload(payload);

    if let Some(k) = key {
        record = record.key(k);
    }

    match producer
        .send(record, Timeout::After(Duration::from_secs(5)))
        .await
    {
        Ok((_partition, _offset)) => {
            debug!(topic = topic, "message published successfully");
            Ok(())
        }
        Err((e, _message)) => {
            error!(topic = topic, error = %e, "failed to publish message");
            Err(AppError::Kafka(format!("Publish error: {}", e)))
        }
    }
}
