use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    #[allow(dead_code)]
    Config(String),
    #[error("Kafka/Redpanda error: {0}")]
    #[allow(dead_code)]
    Kafka(String),
    #[error("DB error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Time parse error: {0}")]
    #[allow(dead_code)]
    Time(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
