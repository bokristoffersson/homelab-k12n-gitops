use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    api::models::plugs::{
        PlugResponse, PlugsListResponse, ScheduleResponse, SchedulesListResponse,
    },
    error::Result,
    repositories::plugs::{
        PowerPlugCreate, PowerPlugToggle, PowerPlugUpdate, ScheduleCreate, ScheduleUpdate,
    },
};

use super::AppState;

// ============================================================================
// Power Plug Handlers
// ============================================================================

/// GET /api/v1/plugs
/// Returns all power plugs
pub async fn get_all_plugs(State(state): State<AppState>) -> Result<Json<PlugsListResponse>> {
    let plugs = state.plugs_repository.get_all().await?;

    Ok(Json(PlugsListResponse { plugs }))
}

/// GET /api/v1/plugs/:plug_id
/// Returns a specific power plug
pub async fn get_plug(
    State(state): State<AppState>,
    Path(plug_id): Path<String>,
) -> Result<Json<PlugResponse>> {
    let plug = state.plugs_repository.get_by_id(&plug_id).await?;

    Ok(Json(PlugResponse {
        plug,
        outbox_id: None,
        outbox_status: None,
    }))
}

/// POST /api/v1/plugs
/// Create a new power plug
pub async fn create_plug(
    State(state): State<AppState>,
    Json(create): Json<PowerPlugCreate>,
) -> Result<(StatusCode, Json<PlugResponse>)> {
    let plug = state.plugs_repository.create(&create).await?;

    Ok((
        StatusCode::CREATED,
        Json(PlugResponse {
            plug,
            outbox_id: None,
            outbox_status: None,
        }),
    ))
}

/// PUT /api/v1/plugs/:plug_id
/// Update a power plug's name
pub async fn update_plug(
    State(state): State<AppState>,
    Path(plug_id): Path<String>,
    Json(update): Json<PowerPlugUpdate>,
) -> Result<Json<PlugResponse>> {
    let plug = state.plugs_repository.update(&plug_id, &update).await?;

    Ok(Json(PlugResponse {
        plug,
        outbox_id: None,
        outbox_status: None,
    }))
}

/// PATCH /api/v1/plugs/:plug_id
/// Toggle plug status (uses outbox pattern for MQTT command)
pub async fn toggle_plug(
    State(state): State<AppState>,
    Path(plug_id): Path<String>,
    Json(toggle): Json<PowerPlugToggle>,
) -> Result<(StatusCode, Json<PlugResponse>)> {
    // Begin transaction
    let mut tx = state.pool.begin().await?;

    // 1. Update plug status (within transaction)
    let plug = crate::repositories::plugs::PlugsRepository::update_status_in_tx(
        &mut tx,
        &plug_id,
        toggle.status,
    )
    .await?;

    // 2. Insert outbox command (within same transaction)
    let outbox_entry = crate::repositories::outbox::OutboxRepository::insert_plug_command_in_tx(
        &mut tx,
        &plug_id,
        toggle.status,
    )
    .await?;

    // 3. Commit transaction (atomic: both succeed or both fail)
    tx.commit().await?;

    // Return 202 Accepted (command is pending, not yet confirmed by device)
    Ok((
        StatusCode::ACCEPTED,
        Json(PlugResponse {
            plug,
            outbox_id: Some(outbox_entry.id),
            outbox_status: Some(outbox_entry.status),
        }),
    ))
}

/// DELETE /api/v1/plugs/:plug_id
/// Delete a power plug
pub async fn delete_plug(
    State(state): State<AppState>,
    Path(plug_id): Path<String>,
) -> Result<StatusCode> {
    state.plugs_repository.delete(&plug_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Schedule Handlers
// ============================================================================

/// GET /api/v1/plugs/:plug_id/schedules
/// Returns all schedules for a plug
pub async fn get_schedules(
    State(state): State<AppState>,
    Path(plug_id): Path<String>,
) -> Result<Json<SchedulesListResponse>> {
    // Verify plug exists
    state.plugs_repository.get_by_id(&plug_id).await?;

    let schedules = state.schedules_repository.get_by_plug_id(&plug_id).await?;

    Ok(Json(SchedulesListResponse { schedules }))
}

/// GET /api/v1/plugs/:plug_id/schedules/:schedule_id
/// Returns a specific schedule
pub async fn get_schedule(
    State(state): State<AppState>,
    Path((plug_id, schedule_id)): Path<(String, i64)>,
) -> Result<Json<ScheduleResponse>> {
    // Verify plug exists
    state.plugs_repository.get_by_id(&plug_id).await?;

    let schedule = state.schedules_repository.get_by_id(schedule_id).await?;

    // Verify schedule belongs to plug
    if schedule.plug_id != plug_id {
        return Err(crate::error::AppError::NotFound(format!(
            "Schedule {} not found for plug {}",
            schedule_id, plug_id
        )));
    }

    Ok(Json(ScheduleResponse { schedule }))
}

/// POST /api/v1/plugs/:plug_id/schedules
/// Create a new schedule for a plug
pub async fn create_schedule(
    State(state): State<AppState>,
    Path(plug_id): Path<String>,
    Json(create): Json<ScheduleCreate>,
) -> Result<(StatusCode, Json<ScheduleResponse>)> {
    // Verify plug exists
    state.plugs_repository.get_by_id(&plug_id).await?;

    let schedule = state.schedules_repository.create(&plug_id, &create).await?;

    Ok((StatusCode::CREATED, Json(ScheduleResponse { schedule })))
}

/// PUT /api/v1/plugs/:plug_id/schedules/:schedule_id
/// Update a schedule
pub async fn update_schedule(
    State(state): State<AppState>,
    Path((plug_id, schedule_id)): Path<(String, i64)>,
    Json(update): Json<ScheduleUpdate>,
) -> Result<Json<ScheduleResponse>> {
    // Verify plug exists
    state.plugs_repository.get_by_id(&plug_id).await?;

    // Verify schedule belongs to plug
    let existing = state.schedules_repository.get_by_id(schedule_id).await?;
    if existing.plug_id != plug_id {
        return Err(crate::error::AppError::NotFound(format!(
            "Schedule {} not found for plug {}",
            schedule_id, plug_id
        )));
    }

    let schedule = state
        .schedules_repository
        .update(schedule_id, &update)
        .await?;

    Ok(Json(ScheduleResponse { schedule }))
}

/// DELETE /api/v1/plugs/:plug_id/schedules/:schedule_id
/// Delete a schedule
pub async fn delete_schedule(
    State(state): State<AppState>,
    Path((plug_id, schedule_id)): Path<(String, i64)>,
) -> Result<StatusCode> {
    // Verify plug exists
    state.plugs_repository.get_by_id(&plug_id).await?;

    // Verify schedule belongs to plug
    let existing = state.schedules_repository.get_by_id(schedule_id).await?;
    if existing.plug_id != plug_id {
        return Err(crate::error::AppError::NotFound(format!(
            "Schedule {} not found for plug {}",
            schedule_id, plug_id
        )));
    }

    state.schedules_repository.delete(schedule_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
