use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    #[allow(dead_code)]
    Config(String),
    #[error("DB error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse error: {0}")]
    #[allow(dead_code)]
    Parse(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
