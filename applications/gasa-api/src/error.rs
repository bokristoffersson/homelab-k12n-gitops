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

    #[error("Not found")]
    NotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Conflict: {0}")]
    Conflict(&'static str),

    #[error("Invalid input: {0}")]
    Validation(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error".to_string(),
                )
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            AppError::Conflict(code) => (StatusCode::CONFLICT, (*code).to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
