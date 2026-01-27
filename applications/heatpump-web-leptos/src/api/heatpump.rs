use super::client::{ApiClient, ApiError};
use crate::models::HeatpumpStatus;

impl ApiClient {
    /// Get the latest heatpump status
    pub async fn get_heatpump_status(&self) -> Result<HeatpumpStatus, ApiError> {
        self.get("/api/v1/heatpump/status").await
    }
}
