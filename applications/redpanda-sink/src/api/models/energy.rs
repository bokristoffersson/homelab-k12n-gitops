use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EnergyLatestResponse {
    pub ts: DateTime<Utc>,
    pub consumption_total_w: Option<f64>,
    pub consumption_total_actual_w: Option<i64>,
    pub consumption_l1_w: Option<f64>,
    pub consumption_l2_w: Option<f64>,
    pub consumption_l3_w: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct HourlyTotalResponse {
    pub total_kwh: f64,
    pub hour_start: DateTime<Utc>,
    pub current_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EnergyHourlyResponse {
    pub hour_start: DateTime<Utc>,
    pub hour_end: DateTime<Utc>,
    pub total_energy_kwh: Option<f64>,
    pub total_energy_l1_kwh: Option<f64>,
    pub total_energy_l2_kwh: Option<f64>,
    pub total_energy_l3_kwh: Option<f64>,
    pub total_energy_actual_kwh: Option<f64>,
    pub measurement_count: i64,
}


