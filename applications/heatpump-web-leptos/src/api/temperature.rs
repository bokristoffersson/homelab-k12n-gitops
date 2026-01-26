use super::client::{ApiClient, ApiError};
use crate::models::{TemperatureLatest, TemperatureReading};

impl ApiClient {
    /// Get the latest temperature readings
    pub async fn get_temperature_latest(&self) -> Result<Vec<TemperatureLatest>, ApiError> {
        self.get("/api/v1/temperature/latest").await
    }

    /// Get temperature history for the last N hours
    pub async fn get_temperature_history(&self, hours: u32) -> Result<Vec<TemperatureReading>, ApiError> {
        self.get(&format!("/api/v1/temperature?hours={}", hours))
            .await
    }
}
