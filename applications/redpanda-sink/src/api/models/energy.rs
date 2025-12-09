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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_latest_response_creation() {
        let response = EnergyLatestResponse {
            ts: Utc::now(),
            consumption_total_w: Some(1500.0),
            consumption_total_actual_w: Some(1500),
            consumption_l1_w: Some(500.0),
            consumption_l2_w: Some(500.0),
            consumption_l3_w: Some(500.0),
        };

        assert!(response.consumption_total_w.is_some());
        assert_eq!(response.consumption_total_w.unwrap(), 1500.0);
        assert_eq!(response.consumption_l1_w.unwrap(), 500.0);
    }

    #[test]
    fn test_energy_latest_response_serialization() {
        let response = EnergyLatestResponse {
            ts: DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            consumption_total_w: Some(2000.0),
            consumption_total_actual_w: Some(2000),
            consumption_l1_w: None,
            consumption_l2_w: None,
            consumption_l3_w: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("2000"));
        assert!(json.contains("2025-01-01"));
    }

    #[test]
    fn test_hourly_total_response_creation() {
        let now = Utc::now();
        let hour_start = now - chrono::Duration::minutes(30);

        let response = HourlyTotalResponse {
            total_kwh: 5.5,
            hour_start,
            current_time: now,
        };

        assert_eq!(response.total_kwh, 5.5);
        assert!(response.current_time >= response.hour_start);
    }

    #[test]
    fn test_hourly_total_response_serialization() {
        let now = Utc::now();
        let response = HourlyTotalResponse {
            total_kwh: 10.25,
            hour_start: now,
            current_time: now,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("10.25"));
    }

    #[test]
    fn test_energy_hourly_response_creation() {
        let hour_start = Utc::now();
        let hour_end = hour_start + chrono::Duration::hours(1);

        let response = EnergyHourlyResponse {
            hour_start,
            hour_end,
            total_energy_kwh: Some(15.5),
            total_energy_l1_kwh: Some(5.0),
            total_energy_l2_kwh: Some(5.0),
            total_energy_l3_kwh: Some(5.5),
            total_energy_actual_kwh: Some(15.5),
            measurement_count: 3600,
        };

        assert_eq!(response.total_energy_kwh.unwrap(), 15.5);
        assert_eq!(response.measurement_count, 3600);
        assert!(response.hour_end > response.hour_start);
    }

    #[test]
    fn test_energy_hourly_response_with_none_values() {
        let hour_start = Utc::now();
        let hour_end = hour_start + chrono::Duration::hours(1);

        let response = EnergyHourlyResponse {
            hour_start,
            hour_end,
            total_energy_kwh: None,
            total_energy_l1_kwh: None,
            total_energy_l2_kwh: None,
            total_energy_l3_kwh: None,
            total_energy_actual_kwh: None,
            measurement_count: 0,
        };

        assert!(response.total_energy_kwh.is_none());
        assert_eq!(response.measurement_count, 0);
    }

    #[test]
    fn test_energy_hourly_response_serialization() {
        let hour_start = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let hour_end = hour_start + chrono::Duration::hours(1);

        let response = EnergyHourlyResponse {
            hour_start,
            hour_end,
            total_energy_kwh: Some(20.0),
            total_energy_l1_kwh: Some(7.0),
            total_energy_l2_kwh: Some(7.0),
            total_energy_l3_kwh: Some(6.0),
            total_energy_actual_kwh: Some(20.0),
            measurement_count: 7200,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("20.0"));
        assert!(json.contains("7200"));
        assert!(json.contains("2025-01-01"));
    }
}
