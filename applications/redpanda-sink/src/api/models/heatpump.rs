use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HeatpumpLatestResponse {
    pub ts: DateTime<Utc>,
    pub device_id: Option<String>,
    pub compressor_on: Option<bool>,
    pub hotwater_production: Option<bool>,
    pub flowlinepump_on: Option<bool>,
    pub brinepump_on: Option<bool>,
    pub aux_heater_3kw_on: Option<bool>,
    pub aux_heater_6kw_on: Option<bool>,
    pub outdoor_temp: Option<f64>,
    pub supplyline_temp: Option<f64>,
    pub returnline_temp: Option<f64>,
    pub hotwater_temp: Option<i64>,
    pub brine_out_temp: Option<i64>,
    pub brine_in_temp: Option<i64>,
}


