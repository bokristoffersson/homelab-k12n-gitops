use super::client::{ApiClient, ApiError};
use crate::models::{EnergyHourly, EnergyLatest, HourlyTotal};

#[allow(dead_code)]
impl ApiClient {
    /// Get the latest energy reading
    pub async fn get_energy_latest(&self) -> Result<EnergyLatest, ApiError> {
        self.get("/api/v1/energy/latest").await
    }

    /// Get the hourly total for the current hour
    pub async fn get_hourly_total(&self) -> Result<HourlyTotal, ApiError> {
        self.get("/api/v1/energy/hourly-total").await
    }

    /// Get hourly energy history for the last N hours
    pub async fn get_energy_hourly(&self, hours: u32) -> Result<Vec<EnergyHourly>, ApiError> {
        self.get(&format!("/api/v1/energy/hourly?hours={}", hours))
            .await
    }
}
