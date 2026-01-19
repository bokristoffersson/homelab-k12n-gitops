use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    api::models::{
        OutboxEntriesResponse, OutboxStatusResponse, SettingResponse, SettingsListResponse,
    },
    error::Result,
    repositories::{outbox::OutboxRepository, settings::SettingPatch, SettingsRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<SettingsRepository>,
    pub outbox_repository: Arc<OutboxRepository>,
    pub pool: PgPool,
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

    Ok(Json(SettingResponse {
        setting,
        outbox_id: None,
        outbox_status: None,
    }))
}

/// PATCH /api/v1/heatpump/settings/:device_id
/// Partially update settings for a device using transactional outbox pattern
pub async fn update_setting(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
    Json(patch): Json<SettingPatch>,
) -> Result<(StatusCode, Json<SettingResponse>)> {
    // Validate patch data
    validate_patch(&patch)?;

    // Begin transaction
    let mut tx = state.pool.begin().await?;

    // 1. Update settings table (within transaction)
    let setting = update_setting_in_tx(&mut tx, &device_id, &patch).await?;

    // 2. Insert outbox command (within same transaction)
    let outbox_entry =
        crate::repositories::outbox::OutboxRepository::insert_in_tx(&mut tx, &device_id, &patch)
            .await?;

    // 3. Commit transaction (atomic: both succeed or both fail)
    tx.commit().await?;

    // Return 202 Accepted (command is pending, not yet confirmed by heatpump)
    Ok((
        StatusCode::ACCEPTED,
        Json(SettingResponse {
            setting,
            outbox_id: Some(outbox_entry.id),
            outbox_status: Some(outbox_entry.status),
        }),
    ))
}

/// Helper: Update setting within a transaction
async fn update_setting_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    device_id: &str,
    patch: &SettingPatch,
) -> Result<crate::repositories::settings::Setting> {
    use crate::error::AppError;

    // Build dynamic UPDATE query based on which fields are present
    let mut query = String::from("UPDATE settings SET updated_at = NOW()");
    let mut bind_count = 1;

    if patch.indoor_target_temp.is_some() {
        bind_count += 1;
        query.push_str(&format!(", indoor_target_temp = ${}", bind_count));
    }
    if patch.mode.is_some() {
        bind_count += 1;
        query.push_str(&format!(", mode = ${}", bind_count));
    }
    if patch.curve.is_some() {
        bind_count += 1;
        query.push_str(&format!(", curve = ${}", bind_count));
    }
    if patch.curve_min.is_some() {
        bind_count += 1;
        query.push_str(&format!(", curve_min = ${}", bind_count));
    }
    if patch.curve_max.is_some() {
        bind_count += 1;
        query.push_str(&format!(", curve_max = ${}", bind_count));
    }
    if patch.curve_plus_5.is_some() {
        bind_count += 1;
        query.push_str(&format!(", curve_plus_5 = ${}", bind_count));
    }
    if patch.curve_zero.is_some() {
        bind_count += 1;
        query.push_str(&format!(", curve_zero = ${}", bind_count));
    }
    if patch.curve_minus_5.is_some() {
        bind_count += 1;
        query.push_str(&format!(", curve_minus_5 = ${}", bind_count));
    }
    if patch.heatstop.is_some() {
        bind_count += 1;
        query.push_str(&format!(", heatstop = ${}", bind_count));
    }
    if patch.integral_setting.is_some() {
        bind_count += 1;
        query.push_str(&format!(", integral_setting = ${}", bind_count));
    }

    query.push_str(" WHERE device_id = $1 RETURNING *");

    let mut query_builder =
        sqlx::query_as::<_, crate::repositories::settings::Setting>(&query).bind(device_id);

    if let Some(val) = patch.indoor_target_temp {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.mode {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.curve {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.curve_min {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.curve_max {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.curve_plus_5 {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.curve_zero {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.curve_minus_5 {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.heatstop {
        query_builder = query_builder.bind(val);
    }
    if let Some(val) = patch.integral_setting {
        query_builder = query_builder.bind(val);
    }

    let setting = query_builder
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Device {} not found", device_id)))?;

    Ok(setting)
}

/// GET /api/v1/heatpump/settings/:device_id/outbox
/// Get outbox entries for a specific device
pub async fn get_outbox_entries_by_device(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
) -> Result<Json<OutboxEntriesResponse>> {
    let entries = state
        .outbox_repository
        .get_by_device_id(&device_id, 10)
        .await?;

    Ok(Json(OutboxEntriesResponse { data: entries }))
}

/// GET /api/v1/heatpump/settings/outbox/:id
/// Get outbox status for a specific command
pub async fn get_outbox_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<OutboxStatusResponse>> {
    let entry = state.outbox_repository.get_by_id(id).await?;

    Ok(Json(OutboxStatusResponse {
        id: entry.id,
        status: entry.status,
        created_at: entry.created_at,
        published_at: entry.published_at,
        confirmed_at: entry.confirmed_at,
        error_message: entry.error_message,
        retry_count: entry.retry_count,
    }))
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
