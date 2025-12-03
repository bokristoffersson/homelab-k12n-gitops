use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("MQTT error: {0}")]
    Mqtt(String),
    #[error("Kafka/Redpanda error: {0}")]
    Kafka(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Time parse error: {0}")]
    Time(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
