use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::models::{HeatpumpListResponse, HeatpumpQueryParams, HeatpumpReading};
use crate::services::HeatpumpService;

#[derive(Deserialize)]
pub struct GetByIdParams {
    device_id: Option<String>,
}

pub async fn list(
    State(service): State<HeatpumpService>,
    Query(params): Query<HeatpumpQueryParams>,
) -> Result<Json<HeatpumpListResponse>> {
    let response = service.list(params).await?;
    Ok(Json(response))
}

pub async fn get_by_id(
    State(service): State<HeatpumpService>,
    Path(ts): Path<String>,
    Query(params): Query<GetByIdParams>,
) -> Result<Json<HeatpumpReading>> {
    let timestamp = DateTime::parse_from_rfc3339(&ts)
        .map_err(|_| AppError::Validation(format!("Invalid timestamp format: {}", ts)))?
        .with_timezone(&Utc);

    let reading = service.get_by_id(timestamp, params.device_id).await?;
    Ok(Json(reading))
}

pub async fn get_latest(
    State(service): State<HeatpumpService>,
    Query(params): Query<GetByIdParams>,
) -> Result<Json<HeatpumpReading>> {
    let reading = service.get_latest(params.device_id).await?;
    Ok(Json(reading))
}

pub async fn health() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok" })),
    )
}

