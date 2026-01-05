use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    api::models::{SettingResponse, SettingsListResponse},
    error::Result,
    repositories::{settings::SettingPatch, SettingsRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<SettingsRepository>,
}

/// GET /api/v1/heatpump/settings
/// Returns all device settings
pub async fn get_all_settings(State(state): State<AppState>) -> Result<Json<SettingsListResponse>> {
    let settings = state.repository.get_all().await?;

    Ok(Json(SettingsListResponse { settings }))
}

/// GET /api/v1/heatpump/settings/:device_id
/// Returns settings for a specific device
pub async fn get_setting_by_device(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
) -> Result<Json<SettingResponse>> {
    let setting = state.repository.get_by_device_id(&device_id).await?;

    Ok(Json(SettingResponse { setting }))
}

/// PATCH /api/v1/heatpump/settings/:device_id
/// Partially update settings for a device
pub async fn update_setting(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
    Json(patch): Json<SettingPatch>,
) -> Result<(StatusCode, Json<SettingResponse>)> {
    // Validate patch data
    validate_patch(&patch)?;

    let setting = state.repository.update(&device_id, &patch).await?;

    Ok((StatusCode::OK, Json(SettingResponse { setting })))
}

/// Validate PATCH request data
fn validate_patch(patch: &SettingPatch) -> Result<()> {
    use crate::error::AppError;

    // Validate temperature range
    if let Some(temp) = patch.indoor_target_temp {
        if !(15.0..=30.0).contains(&temp) {
            return Err(AppError::InvalidInput(
                "indoor_target_temp must be between 15 and 30".to_string(),
            ));
        }
    }

    // Validate mode range
    if let Some(mode) = patch.mode {
        if !(0..=3).contains(&mode) {
            return Err(AppError::InvalidInput(
                "mode must be between 0 and 3".to_string(),
            ));
        }
    }

    Ok(())
}
