use crate::api::models::energy::{
    EnergyHourlyResponse, EnergyLatestResponse, EnergySummaryResponse, HourlyTotalResponse,
};
use crate::db::DbPool;
use crate::repositories::EnergyRepository;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub async fn get_latest(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
) -> Result<Json<EnergyLatestResponse>, StatusCode> {
    let reading = EnergyRepository::get_latest(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(EnergyLatestResponse {
        ts: reading.ts,
        consumption_total_w: reading.consumption_total_w,
        consumption_total_actual_w: reading.consumption_total_actual_w,
        consumption_l1_actual_w: reading.consumption_l1_actual_w,
        consumption_l2_actual_w: reading.consumption_l2_actual_w,
        consumption_l3_actual_w: reading.consumption_l3_actual_w,
    }))
}

pub async fn get_hourly_total(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
) -> Result<Json<HourlyTotalResponse>, StatusCode> {
    let now = Utc::now();
    // Align to hour boundary using same origin as aggregate
    // Origin is '2000-01-01 00:00:00+00', so we need to align relative to that
    let origin = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00")
        .unwrap()
        .with_timezone(&Utc);
    let seconds_since_origin = (now - origin).num_seconds();
    let hours_since_origin = seconds_since_origin / 3600;
    let hour_start = origin + chrono::Duration::hours(hours_since_origin);

    let total_kwh = EnergyRepository::get_hourly_total(&pool, hour_start)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(HourlyTotalResponse {
        total_kwh,
        hour_start,
        current_time: now,
    }))
}

pub async fn get_history(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EnergyHourlyResponse>>, StatusCode> {
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

    let readings = EnergyRepository::get_hourly_history(&pool, from, to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<EnergyHourlyResponse> = readings
        .into_iter()
        .map(|r| EnergyHourlyResponse {
            hour_start: r.hour_start,
            hour_end: r.hour_end,
            total_energy_kwh: r.total_energy_kwh,
            total_energy_l1_kwh: r.total_energy_l1_kwh,
            total_energy_l2_kwh: r.total_energy_l2_kwh,
            total_energy_l3_kwh: r.total_energy_l3_kwh,
            total_energy_actual_kwh: r.total_energy_actual_kwh,
            measurement_count: r.measurement_count,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_daily_summary(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EnergySummaryResponse>>, StatusCode> {
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

    let summaries = EnergyRepository::get_daily_summary(&pool, from, to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<EnergySummaryResponse> = summaries
        .into_iter()
        .map(|s| EnergySummaryResponse {
            day_start: s.day_start,
            day_end: s.day_end,
            month_start: s.month_start,
            month_end: s.month_end,
            year_start: s.year_start,
            year_end: s.year_end,
            energy_consumption_kwh: s.energy_consumption_w,
            measurement_count: s.measurement_count,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_monthly_summary(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EnergySummaryResponse>>, StatusCode> {
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

    let summaries = EnergyRepository::get_monthly_summary(&pool, from, to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<EnergySummaryResponse> = summaries
        .into_iter()
        .map(|s| EnergySummaryResponse {
            day_start: s.day_start,
            day_end: s.day_end,
            month_start: s.month_start,
            month_end: s.month_end,
            year_start: s.year_start,
            year_end: s.year_end,
            energy_consumption_kwh: s.energy_consumption_w,
            measurement_count: s.measurement_count,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_yearly_summary(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EnergySummaryResponse>>, StatusCode> {
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

    let summaries = EnergyRepository::get_yearly_summary(&pool, from, to)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let responses: Vec<EnergySummaryResponse> = summaries
        .into_iter()
        .map(|s| EnergySummaryResponse {
            day_start: s.day_start,
            day_end: s.day_end,
            month_start: s.month_start,
            month_end: s.month_end,
            year_start: s.year_start,
            year_end: s.year_end,
            energy_consumption_kwh: s.energy_consumption_w,
            measurement_count: s.measurement_count,
        })
        .collect();

    Ok(Json(responses))
}

// INTENTIONAL VIOLATION: This endpoint writes to the database, violating the read-only principle
pub async fn reset_energy_data(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
) -> Result<StatusCode, StatusCode> {
    // This is a database write operation - should be caught by arch review
    sqlx::query!("DELETE FROM telemetry.energy WHERE ts < NOW() - INTERVAL '7 days'")
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
