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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heatpump_latest_response_creation() {
        let response = HeatpumpLatestResponse {
            ts: Utc::now(),
            device_id: Some("hp-01".to_string()),
            compressor_on: Some(true),
            hotwater_production: Some(false),
            flowlinepump_on: Some(true),
            brinepump_on: Some(true),
            aux_heater_3kw_on: Some(false),
            aux_heater_6kw_on: Some(false),
            outdoor_temp: Some(5.5),
            supplyline_temp: Some(35.0),
            returnline_temp: Some(30.0),
            hotwater_temp: Some(45),
            brine_out_temp: Some(8),
            brine_in_temp: Some(6),
        };

        assert_eq!(response.device_id.as_ref().unwrap(), "hp-01");
        assert!(response.compressor_on.unwrap());
        assert_eq!(response.outdoor_temp.unwrap(), 5.5);
    }

    #[test]
    fn test_heatpump_latest_response_with_none_values() {
        let response = HeatpumpLatestResponse {
            ts: Utc::now(),
            device_id: None,
            compressor_on: None,
            hotwater_production: None,
            flowlinepump_on: None,
            brinepump_on: None,
            aux_heater_3kw_on: None,
            aux_heater_6kw_on: None,
            outdoor_temp: None,
            supplyline_temp: None,
            returnline_temp: None,
            hotwater_temp: None,
            brine_out_temp: None,
            brine_in_temp: None,
        };

        assert!(response.device_id.is_none());
        assert!(response.compressor_on.is_none());
        assert!(response.outdoor_temp.is_none());
    }

    #[test]
    fn test_heatpump_latest_response_serialization() {
        let response = HeatpumpLatestResponse {
            ts: DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            device_id: Some("device-123".to_string()),
            compressor_on: Some(true),
            hotwater_production: Some(true),
            flowlinepump_on: Some(false),
            brinepump_on: Some(true),
            aux_heater_3kw_on: Some(false),
            aux_heater_6kw_on: Some(false),
            outdoor_temp: Some(-5.0),
            supplyline_temp: Some(40.0),
            returnline_temp: Some(35.0),
            hotwater_temp: Some(50),
            brine_out_temp: Some(10),
            brine_in_temp: Some(8),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("device-123"));
        assert!(json.contains("true"));
        assert!(json.contains("-5.0"));
        assert!(json.contains("50"));
    }

    #[test]
    fn test_heatpump_latest_response_all_boolean_states() {
        // Test all possible boolean combinations
        let combinations = vec![
            (true, true, true, true),
            (true, false, false, false),
            (false, true, true, false),
            (false, false, false, false),
        ];

        for (compressor, hotwater, flowline, brine) in combinations {
            let response = HeatpumpLatestResponse {
                ts: Utc::now(),
                device_id: Some("test".to_string()),
                compressor_on: Some(compressor),
                hotwater_production: Some(hotwater),
                flowlinepump_on: Some(flowline),
                brinepump_on: Some(brine),
                aux_heater_3kw_on: Some(false),
                aux_heater_6kw_on: Some(false),
                outdoor_temp: Some(0.0),
                supplyline_temp: Some(0.0),
                returnline_temp: Some(0.0),
                hotwater_temp: Some(0),
                brine_out_temp: Some(0),
                brine_in_temp: Some(0),
            };

            assert_eq!(response.compressor_on.unwrap(), compressor);
            assert_eq!(response.hotwater_production.unwrap(), hotwater);
            assert_eq!(response.flowlinepump_on.unwrap(), flowline);
            assert_eq!(response.brinepump_on.unwrap(), brine);
        }
    }
}
