use super::client::{ApiClient, ApiError};
use crate::models::{HeatpumpSetting, SettingPatch, SettingsResponse};

impl ApiClient {
    /// Get all heatpump settings
    pub async fn get_settings(&self) -> Result<SettingsResponse, ApiError> {
        self.get("/api/v1/settings").await
    }

    /// Get settings for a specific device
    pub async fn get_device_settings(&self, device_id: &str) -> Result<HeatpumpSetting, ApiError> {
        self.get(&format!("/api/v1/settings/{}", device_id)).await
    }

    /// Update settings for a specific device
    pub async fn update_settings(
        &self,
        device_id: &str,
        patch: &SettingPatch,
    ) -> Result<HeatpumpSetting, ApiError> {
        self.patch(&format!("/api/v1/settings/{}", device_id), patch)
            .await
    }
}
