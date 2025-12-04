use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(String),
    #[error("Kafka/Redpanda error: {0}")]
    Kafka(String),
    #[error("DB error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Time parse error: {0}")]
    Time(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

