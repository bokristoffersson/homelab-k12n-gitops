use crate::api::models::temperature::{TemperatureLatest, TemperatureReading};
use crate::db::DbPool;
use crate::repositories::TemperatureRepository;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use std::collections::HashMap;

pub async fn get_latest(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Option<TemperatureLatest>>, StatusCode> {
    let location = params.get("location").ok_or(StatusCode::BAD_REQUEST)?;

    let reading = TemperatureRepository::get_latest_by_location(&pool, location)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(reading.map(|r| r.into())))
}

pub async fn get_all_latest(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
) -> Result<Json<Vec<TemperatureLatest>>, StatusCode> {
    let readings = TemperatureRepository::get_all_latest(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(readings.into_iter().map(|r| r.into()).collect()))
}

pub async fn get_history(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<TemperatureReading>>, StatusCode> {
    let location = params.get("location").ok_or(StatusCode::BAD_REQUEST)?;

    let hours: i32 = params
        .get("hours")
        .and_then(|h| h.parse().ok())
        .unwrap_or(24);

    let readings = TemperatureRepository::get_history(&pool, location, hours)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(readings.into_iter().map(|r| r.into()).collect()))
}
