use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(e) => {
                // Log full error for debugging
                tracing::error!("Database error: {:?}", e);
                
                // Provide more descriptive error messages based on error type
                let user_message = match e {
                    sqlx::Error::Database(db_err) => {
                        let msg = db_err.message();
                        if msg.contains("does not exist") || msg.contains("relation") {
                            "Database table or schema does not exist. Please run migrations."
                        } else if msg.contains("connection") || msg.contains("Connection") {
                            "Database connection failed. Please check database configuration."
                        } else if msg.contains("database") && msg.contains("does not exist") {
                            "Database does not exist. Please check database configuration."
                        } else {
                            "Database query error. Please check logs for details."
                        }
                    }
                    sqlx::Error::PoolClosed => "Database connection pool is closed.",
                    sqlx::Error::PoolTimedOut => "Database connection timeout. Please try again.",
                    sqlx::Error::Io(_) => "Database I/O error. Please check database availability.",
                    _ => "Database error occurred. Please check logs for details.",
                };
                
                (StatusCode::INTERNAL_SERVER_ERROR, user_message)
            }
            AppError::Kafka(ref e) => {
                tracing::error!("Kafka error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Kafka error")
            }
            AppError::Serialization(ref e) => {
                tracing::error!("Serialization error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid data format")
            }
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
