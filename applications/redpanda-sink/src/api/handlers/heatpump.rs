use crate::api::models::heatpump::{HeatpumpDailySummaryResponse, HeatpumpLatestResponse};
use crate::db::DbPool;
use crate::repositories::HeatpumpRepository;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{error, warn};
use sqlx::Error as SqlxError;

pub async fn get_latest(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<HeatpumpLatestResponse>, StatusCode> {
    let device_id = params.get("device_id").map(|s| s.as_str());
    
    tracing::debug!(device_id = ?device_id, "fetching latest heatpump reading");

    let reading = HeatpumpRepository::get_latest(&pool, device_id)
        .await
        .map_err(|e| {
            if let crate::error::AppError::Db(db_err) = &e {
                if matches!(db_err, SqlxError::RowNotFound) {
                    warn!(device_id = ?device_id, "no heatpump data found");
                    return StatusCode::NOT_FOUND;
                }
                error!(error = %db_err, device_id = ?device_id, "database error fetching heatpump data");
            } else {
                error!(error = %e, device_id = ?device_id, "error fetching heatpump data");
            }
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::debug!(ts = ?reading.ts, "successfully fetched heatpump reading");

    Ok(Json(HeatpumpLatestResponse {
        ts: reading.ts,
        device_id: None, // device_id column doesn't exist in heatpump table
        compressor_on: reading.compressor_on,
        hotwater_production: reading.hotwater_production,
        flowlinepump_on: reading.flowlinepump_on,
        brinepump_on: reading.brinepump_on,
        aux_heater_3kw_on: reading.aux_heater_3kw_on,
        aux_heater_6kw_on: reading.aux_heater_6kw_on,
        outdoor_temp: reading.outdoor_temp,
        supplyline_temp: reading.supplyline_temp,
        returnline_temp: reading.returnline_temp,
        hotwater_temp: reading.hotwater_temp,
        brine_out_temp: reading.brine_out_temp,
        brine_in_temp: reading.brine_in_temp,
    }))
}

pub async fn get_daily_summary(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<HeatpumpDailySummaryResponse>>, StatusCode> {
    let from = params
        .get("from")
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .ok_or(StatusCode::BAD_REQUEST)?;

    let to = params
        .get("to")
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let summaries = HeatpumpRepository::get_daily_summary(&pool, from, to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<HeatpumpDailySummaryResponse> = summaries
        .into_iter()
        .map(|s| HeatpumpDailySummaryResponse {
            day: s.day,
            daily_runtime_compressor_increase: s.daily_runtime_compressor_increase,
            daily_runtime_hotwater_increase: s.daily_runtime_hotwater_increase,
            daily_runtime_3kw_increase: s.daily_runtime_3kw_increase,
            daily_runtime_6kw_increase: s.daily_runtime_6kw_increase,
            avg_outdoor_temp: s.avg_outdoor_temp,
            avg_supplyline_temp: s.avg_supplyline_temp,
            avg_returnline_temp: s.avg_returnline_temp,
            avg_hotwater_temp: s.avg_hotwater_temp,
            avg_brine_out_temp: s.avg_brine_out_temp,
            avg_brine_in_temp: s.avg_brine_in_temp,
        })
        .collect();

    Ok(Json(responses))
}
