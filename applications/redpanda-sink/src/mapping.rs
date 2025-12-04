use crate::config::{Pipeline, TimestampConfig};
use crate::error::AppError;
use chrono::{DateTime, TimeZone, Utc};
use jsonpath_lib as jsonpath;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Row {
    pub ts: chrono::DateTime<Utc>,
    pub tags: BTreeMap<String, String>,
    pub fields: BTreeMap<String, FieldValue>,
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    F64(f64),
    I64(i64),
    Bool(bool),
    Text(String),
}

pub fn topic_matches(filter: &str, topic: &str) -> bool {
    let fseg: Vec<&str> = filter.split('/').collect();
    let tseg: Vec<&str> = topic.split('/').collect();
    for (i, f) in fseg.iter().enumerate() {
        match *f {
            "#" => return true,
            "+" => {
                if i >= tseg.len() {
                    return false;
                }
            }
            _ => {
                if i >= tseg.len() || *f != tseg[i] {
                    return false;
                }
            }
        }
    }
    fseg.len() == tseg.len()
}

pub fn extract_row(p: &Pipeline, _topic: &str, payload: &[u8]) -> Result<Row, AppError> {
    let json: Value = serde_json::from_slice(payload)?;
    let ts = extract_ts(&p.timestamp, &json)?;

    let mut tags = BTreeMap::new();
    for (col, jpath) in &p.tags {
        if let Some(v) = first_jsonpath(jpath, &json) {
            let s = stringify_json(&v);
            tags.insert(col.clone(), s);
        }
    }

    let mut fields = BTreeMap::new();

    // Handle regular fields
    for (col, fc) in &p.fields {
        if let Some(v) = first_jsonpath(&fc.path, &json) {
            if fc.r#type == "nested" {
                // Handle nested object extraction
                if let Some(attributes) = &fc.attributes {
                    if let Some(obj) = v.as_object() {
                        for (attr_name, output_col) in attributes {
                            if let Some(attr_value) = obj.get(attr_name) {
                                if let Some(converted) = cast_value(attr_value, "float") {
                                    fields.insert(output_col.clone(), converted);
                                }
                            }
                        }
                    }
                }
            } else {
                // Handle regular field types
                if let Some(converted) = cast_value(&v, &fc.r#type) {
                    fields.insert(col.clone(), converted);
                }
            }
        }
    }

    // Handle bit flag fields
    if let Some(bit_flags) = &p.bit_flags {
        for bit_flag_config in bit_flags {
            if let Some(byte_value) = first_jsonpath(&bit_flag_config.source_path, &json) {
                if let Some(byte) = byte_value.as_u64() {
                    if byte <= 255 {
                        let flags = parse_byte_flags(byte as u8, &bit_flag_config.flags);
                        for (flag_name, flag_value) in flags {
                            fields.insert(flag_name, FieldValue::Bool(flag_value));
                        }
                    }
                }
            }
        }
    }

    Ok(Row { ts, tags, fields })
}

/// Extracts boolean flags from a byte value based on bit positions
///
/// # Arguments
/// * `value` - The byte value (0-255) containing the 8 bit flags
/// * `bit_names` - Map of bit positions (0-7) to field names
///
/// # Returns
/// A BTreeMap mapping each field name to its boolean value
fn parse_byte_flags(value: u8, bit_names: &BTreeMap<u8, String>) -> BTreeMap<String, bool> {
    let mut result = BTreeMap::new();

    for (bit_position, field_name) in bit_names {
        if *bit_position < 8 {
            let is_set = (value & (1 << bit_position)) != 0;
            result.insert(field_name.clone(), is_set);
        }
    }

    result
}

fn extract_ts(tc: &TimestampConfig, json: &Value) -> Result<DateTime<Utc>, AppError> {
    if let Some(path) = &tc.path {
        if let Some(v) = first_jsonpath(path, json) {
            match tc.format.as_str() {
                "rfc3339" => {
                    let s = stringify_json(&v);
                    let dt = DateTime::parse_from_rfc3339(&s)
                        .map_err(|e| AppError::Time(format!("rfc3339 parse: {}", e)))?;
                    return Ok(dt.with_timezone(&Utc));
                }
                "unix_ms" => {
                    let ms = v
                        .as_i64()
                        .ok_or_else(|| AppError::Time("unix_ms not i64".into()))?;
                    return Utc
                        .timestamp_millis_opt(ms)
                        .single()
                        .ok_or_else(|| AppError::Time("unix_ms out of range".into()));
                }
                "unix_s" => {
                    let s = v
                        .as_i64()
                        .ok_or_else(|| AppError::Time("unix_s not i64".into()))?;
                    return Utc
                        .timestamp_opt(s, 0)
                        .single()
                        .ok_or_else(|| AppError::Time("unix_s out of range".into()));
                }
                "iso8601" => {
                    let s = stringify_json(&v);
                    let naive_dt = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
                        .map_err(|e| AppError::Time(format!("iso8601 parse: {}", e)))?;
                    return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
                }
                other => return Err(AppError::Time(format!("unknown ts format: {}", other))),
            }
        }
    }
    if tc.use_now {
        Ok(Utc::now())
    } else {
        Err(AppError::Time("timestamp missing".into()))
    }
}

fn first_jsonpath(path: &str, json: &Value) -> Option<Value> {
    jsonpath::select(json, path)
        .ok()
        .and_then(|v| v.into_iter().next().cloned())
}

fn stringify_json(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn cast_value(v: &Value, tpe: &str) -> Option<FieldValue> {
    match tpe {
        "float" => v.as_f64().map(FieldValue::F64),
        "int" => v.as_i64().map(FieldValue::I64),
        "bool" => v.as_bool().map(FieldValue::Bool),
        "text" => Some(FieldValue::Text(stringify_json(v))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BitFlagConfig, FieldConfig, Pipeline, TimestampConfig};
    use crate::mapping::{extract_row, topic_matches, FieldValue};
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_byte_flags() {
        let mut bit_names = BTreeMap::new();
        bit_names.insert(0, "compressor_on".to_string());
        bit_names.insert(1, "heating_mode".to_string());
        bit_names.insert(2, "hot_water_mode".to_string());
        bit_names.insert(4, "circulation_pump".to_string());

        // Test value: 0b00010101 (bits 0, 2, 4 set)
        let result = parse_byte_flags(0b00010101, &bit_names);

        assert_eq!(result.get("compressor_on"), Some(&true));
        assert_eq!(result.get("heating_mode"), Some(&false));
        assert_eq!(result.get("hot_water_mode"), Some(&true));
        assert_eq!(result.get("circulation_pump"), Some(&true));
    }

    #[test]
    fn test_topic_matches() {
        assert!(topic_matches("a/+/c", "a/b/c"));
        assert!(topic_matches("a/#", "a/b/c/d"));
        assert!(topic_matches(
            "home/sensors/+/state",
            "home/sensors/kitchen/state"
        ));
        assert!(!topic_matches("a/+/c", "a/b/c/d"));
        assert!(!topic_matches("a/b/c", "a/b"));
    }

    #[test]
    fn test_extract_row() {
        let mut tags = BTreeMap::new();
        tags.insert("device_id".into(), "$.device".into());

        let mut fields = BTreeMap::new();
        fields.insert(
            "temperature_c".into(),
            FieldConfig {
                path: "$.temperature".into(),
                r#type: "float".into(),
                attributes: None,
            },
        );
        fields.insert(
            "power_w".into(),
            FieldConfig {
                path: "$.power".into(),
                r#type: "int".into(),
                attributes: None,
            },
        );

        let p = Pipeline {
            name: "t".into(),
            topic: "x".into(),
            table: "telemetry".into(),
            data_type: "timeseries".into(),
            upsert_key: None,
            timestamp: TimestampConfig {
                path: None,
                format: "rfc3339".into(),
                use_now: true,
            },
            tags,
            fields,
            bit_flags: None,
            store_interval: None,
        };

        let payload = json!({
            "device": "hp-01",
            "temperature": 21.5,
            "power": 950
        })
        .to_string();

        let row = extract_row(&p, "x", payload.as_bytes()).unwrap();
        assert_eq!(row.tags.get("device_id").unwrap(), "hp-01");

        match row.fields.get("temperature_c").unwrap() {
            FieldValue::F64(v) => assert!((*v - 21.5).abs() < 1e-9),
            _ => panic!("Expected F64"),
        }

        match row.fields.get("power_w").unwrap() {
            FieldValue::I64(v) => assert_eq!(*v, 950),
            _ => panic!("Expected I64"),
        }
    }

    #[test]
    fn test_extract_row_with_bit_flags() {
        let mut tags = BTreeMap::new();
        tags.insert("device_id".into(), "$.device".into());

        let mut fields = BTreeMap::new();
        fields.insert(
            "temperature_c".into(),
            FieldConfig {
                path: "$.temperature".into(),
                r#type: "float".into(),
                attributes: None,
            },
        );

        // Configure bit flags
        let mut status_flags = BTreeMap::new();
        status_flags.insert(0, "compressor_on".into());
        status_flags.insert(1, "heating_mode".into());
        status_flags.insert(2, "hot_water_mode".into());
        status_flags.insert(4, "circulation_pump".into());

        let bit_flag_config = BitFlagConfig {
            source_path: "$.status_byte".into(),
            flags: status_flags,
        };

        let p = Pipeline {
            name: "t".into(),
            topic: "x".into(),
            table: "telemetry".into(),
            data_type: "timeseries".into(),
            upsert_key: None,
            timestamp: TimestampConfig {
                path: None,
                format: "rfc3339".into(),
                use_now: true,
            },
            tags,
            fields,
            bit_flags: Some(vec![bit_flag_config]),
            store_interval: None,
        };

        // Test with status_byte = 21 (0b00010101 - bits 0, 2, 4 set)
        let payload = json!({
            "device": "hp-01",
            "temperature": 21.5,
            "status_byte": 21
        })
        .to_string();

        let row = extract_row(&p, "x", payload.as_bytes()).unwrap();

        // Check regular fields
        assert_eq!(row.tags.get("device_id").unwrap(), "hp-01");
        match row.fields.get("temperature_c").unwrap() {
            FieldValue::F64(v) => assert!((*v - 21.5).abs() < 1e-9),
            _ => panic!("Expected F64"),
        }

        // Check bit flags
        match row.fields.get("compressor_on").unwrap() {
            FieldValue::Bool(v) => assert!(*v),
            _ => panic!("Expected Bool"),
        }

        match row.fields.get("heating_mode").unwrap() {
            FieldValue::Bool(v) => assert!(!(*v)),
            _ => panic!("Expected Bool"),
        }

        match row.fields.get("hot_water_mode").unwrap() {
            FieldValue::Bool(v) => assert!(*v),
            _ => panic!("Expected Bool"),
        }

        match row.fields.get("circulation_pump").unwrap() {
            FieldValue::Bool(v) => assert!(*v),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_extract_row_with_iso8601_timestamp() {
        let mut tags = BTreeMap::new();
        tags.insert("device_id".into(), "$.device".into());

        let mut fields = BTreeMap::new();
        fields.insert(
            "power_w".into(),
            FieldConfig {
                path: "$.power".into(),
                r#type: "int".into(),
                attributes: None,
            },
        );

        let p = Pipeline {
            name: "energy_meter".into(),
            topic: "energy/+/telemetry".into(),
            table: "telemetry".into(),
            data_type: "timeseries".into(),
            upsert_key: None,
            timestamp: TimestampConfig {
                path: Some("$.timestamp".into()),
                format: "iso8601".into(),
                use_now: false,
            },
            tags,
            fields,
            bit_flags: None,
            store_interval: None,
        };

        let payload = json!({
            "device": "em-01",
            "timestamp": "2025-10-25T18:48:26",
            "power": 1500
        })
        .to_string();

        let row = extract_row(&p, "energy/meter/telemetry", payload.as_bytes()).unwrap();

        // Check that timestamp was parsed correctly (should be UTC)
        assert_eq!(
            row.ts.format("%Y-%m-%dT%H:%M:%S").to_string(),
            "2025-10-25T18:48:26"
        );
        assert_eq!(row.tags.get("device_id").unwrap(), "em-01");

        match row.fields.get("power_w").unwrap() {
            FieldValue::I64(v) => assert_eq!(*v, 1500),
            _ => panic!("Expected I64"),
        }
    }

    #[test]
    fn test_extract_row_with_nested_json() {
        let mut tags = BTreeMap::new();
        tags.insert("device_id".into(), "$.device".into());

        let mut fields = BTreeMap::new();
        fields.insert(
            "power_w".into(),
            FieldConfig {
                path: "$.power".into(),
                r#type: "int".into(),
                attributes: None,
            },
        );

        // Configure nested object extraction for energy consumption
        let mut consumption_attrs = BTreeMap::new();
        consumption_attrs.insert("total".into(), "consumption_total_w".into());
        consumption_attrs.insert("L1".into(), "consumption_l1_w".into());
        consumption_attrs.insert("L2".into(), "consumption_l2_w".into());
        consumption_attrs.insert("L3".into(), "consumption_l3_w".into());

        fields.insert(
            "activeActualConsumption".into(),
            FieldConfig {
                path: "$.activeActualConsumption".into(),
                r#type: "nested".into(),
                attributes: Some(consumption_attrs),
            },
        );

        let p = Pipeline {
            name: "energy_meter".into(),
            topic: "energy/+/telemetry".into(),
            table: "telemetry".into(),
            data_type: "timeseries".into(),
            upsert_key: None,
            timestamp: TimestampConfig {
                path: Some("$.timestamp".into()),
                format: "iso8601".into(),
                use_now: false,
            },
            tags,
            fields,
            bit_flags: None,
            store_interval: None,
        };

        let payload = json!({
            "device": "em-01",
            "timestamp": "2025-10-25T18:48:26",
            "power": 1500,
            "activeActualConsumption": {
                "total": 622,
                "L1": 299,
                "L2": 194,
                "L3": 128
            }
        })
        .to_string();

        let row = extract_row(&p, "energy/meter/telemetry", payload.as_bytes()).unwrap();

        // Check regular fields
        assert_eq!(row.tags.get("device_id").unwrap(), "em-01");
        match row.fields.get("power_w").unwrap() {
            FieldValue::I64(v) => assert_eq!(*v, 1500),
            _ => panic!("Expected I64"),
        }

        // Check nested object fields
        match row.fields.get("consumption_total_w").unwrap() {
            FieldValue::F64(v) => assert_eq!(*v, 622.0),
            _ => panic!("Expected F64"),
        }

        match row.fields.get("consumption_l1_w").unwrap() {
            FieldValue::F64(v) => assert_eq!(*v, 299.0),
            _ => panic!("Expected F64"),
        }

        match row.fields.get("consumption_l2_w").unwrap() {
            FieldValue::F64(v) => assert_eq!(*v, 194.0),
            _ => panic!("Expected F64"),
        }

        match row.fields.get("consumption_l3_w").unwrap() {
            FieldValue::F64(v) => assert_eq!(*v, 128.0),
            _ => panic!("Expected F64"),
        }
    }

    #[test]
    fn test_extract_row_with_multiple_bit_flag_sources() {
        let fields = BTreeMap::new();

        // Configure status byte flags
        let mut status_flags = BTreeMap::new();
        status_flags.insert(0, "compressor_on".into());
        status_flags.insert(1, "heating_mode".into());

        // Configure alarm byte flags
        let mut alarm_flags = BTreeMap::new();
        alarm_flags.insert(0, "high_pressure_alarm".into());
        alarm_flags.insert(1, "low_pressure_alarm".into());

        let p = Pipeline {
            name: "t".into(),
            topic: "x".into(),
            table: "telemetry".into(),
            data_type: "timeseries".into(),
            upsert_key: None,
            timestamp: TimestampConfig {
                path: None,
                format: "rfc3339".into(),
                use_now: true,
            },
            tags: BTreeMap::new(),
            fields,
            bit_flags: Some(vec![
                BitFlagConfig {
                    source_path: "$.status_byte".into(),
                    flags: status_flags,
                },
                BitFlagConfig {
                    source_path: "$.alarm_byte".into(),
                    flags: alarm_flags,
                },
            ]),
            store_interval: None,
        };

        // status_byte = 1 (0b00000001 - bit 0 set)
        // alarm_byte = 2 (0b00000010 - bit 1 set)
        let payload = json!({
            "status_byte": 1,
            "alarm_byte": 2
        })
        .to_string();

        let row = extract_row(&p, "x", payload.as_bytes()).unwrap();

        // Check status flags
        match row.fields.get("compressor_on").unwrap() {
            FieldValue::Bool(v) => assert!(*v),
            _ => panic!("Expected Bool"),
        }

        match row.fields.get("heating_mode").unwrap() {
            FieldValue::Bool(v) => assert!(!(*v)),
            _ => panic!("Expected Bool"),
        }

        // Check alarm flags
        match row.fields.get("high_pressure_alarm").unwrap() {
            FieldValue::Bool(v) => assert!(!(*v)),
            _ => panic!("Expected Bool"),
        }

        match row.fields.get("low_pressure_alarm").unwrap() {
            FieldValue::Bool(v) => assert!(*v),
            _ => panic!("Expected Bool"),
        }
    }
}
