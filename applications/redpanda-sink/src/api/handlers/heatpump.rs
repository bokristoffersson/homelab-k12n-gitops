use axum::{extract::{Query, State}, http::StatusCode, response::Json};
use crate::api::models::heatpump::HeatpumpLatestResponse;
use crate::db::DbPool;
use crate::repositories::HeatpumpRepository;
use std::collections::HashMap;

pub async fn get_latest(
    State((pool, _config)): State<(DbPool, crate::config::Config)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<HeatpumpLatestResponse>, StatusCode> {
    let device_id = params.get("device_id").map(|s| s.as_str());
    
    let reading = HeatpumpRepository::get_latest(&pool, device_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(HeatpumpLatestResponse {
        ts: reading.ts,
        device_id: reading.device_id,
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



