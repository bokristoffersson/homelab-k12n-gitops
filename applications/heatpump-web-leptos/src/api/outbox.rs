use super::client::{ApiClient, ApiError};
use crate::models::OutboxResponse;

impl ApiClient {
    /// Get pending outbox entries
    pub async fn get_outbox(&self) -> Result<OutboxResponse, ApiError> {
        self.get("/api/v1/outbox").await
    }
}
