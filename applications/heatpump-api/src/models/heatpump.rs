use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HeatpumpReading {
    pub ts: DateTime<Utc>,
    pub device_id: Option<String>,
    pub room: Option<String>,
    pub outdoor_temp: Option<f64>,
    pub supplyline_temp: Option<f64>,
    pub returnline_temp: Option<f64>,
    pub hotwater_temp: Option<i64>,
    pub brine_out_temp: Option<i64>,
    pub brine_in_temp: Option<i64>,
    pub integral: Option<i64>,
    pub flowlinepump_speed: Option<i64>,
    pub brinepump_speed: Option<i64>,
    pub runtime_compressor: Option<i64>,
    pub runtime_hotwater: Option<i64>,
    pub runtime_3kw: Option<i64>,
    pub runtime_6kw: Option<i64>,
    pub brinepump_on: Option<bool>,
    pub compressor_on: Option<bool>,
    pub flowlinepump_on: Option<bool>,
    pub hotwater_production: Option<bool>,
    pub circulation_pump: Option<bool>,
    pub aux_heater_3kw_on: Option<bool>,
    pub aux_heater_6kw_on: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatpumpQueryParams {
    pub device_id: Option<String>,
    pub room: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for HeatpumpQueryParams {
    fn default() -> Self {
        Self {
            device_id: None,
            room: None,
            limit: Some(100),
            offset: Some(0),
            start_time: None,
            end_time: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatpumpListResponse {
    pub data: Vec<HeatpumpReading>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

