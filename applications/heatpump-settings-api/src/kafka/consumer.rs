use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message,
};
use serde_json::Value;
use std::time::Duration;
use tokio::time;

use crate::{
    config::KafkaConfig,
    repositories::{settings::SettingUpdate, SettingsRepository},
};

pub struct KafkaConsumerService {
    consumer: StreamConsumer,
    topic: String,
    repository: SettingsRepository,
}

impl KafkaConsumerService {
    pub fn new(config: &KafkaConfig, repository: SettingsRepository) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &config.consumer_group)
            .set("bootstrap.servers", config.brokers.join(","))
            .set("enable.auto.commit", config.enable_auto_commit.to_string())
            .set(
                "auto.commit.interval.ms",
                config.auto_commit_interval_ms.to_string(),
            )
            .set("session.timeout.ms", config.session_timeout_ms.to_string())
            .set("auto.offset.reset", &config.auto_offset_reset)
            .create()?;

        consumer.subscribe(&[&config.topic])?;

        tracing::info!(
            "Kafka consumer initialized for topic: {}, group: {}",
            config.topic,
            config.consumer_group
        );

        Ok(Self {
            consumer,
            topic: config.topic.clone(),
            repository,
        })
    }

    pub async fn run(self) {
        tracing::info!("Starting Kafka consumer for topic: {}", self.topic);

        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match self.process_message(payload).await {
                            Ok(_) => {
                                tracing::debug!(
                                    "Successfully processed message from partition {} offset {}",
                                    message.partition(),
                                    message.offset()
                                );
                            }
                            Err(e) => {
                                tracing::error!("Error processing message: {:?}. Continuing...", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka error: {:?}. Retrying in 5s...", e);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn process_message(&self, payload: &[u8]) -> anyhow::Result<()> {
        // Parse JSON message
        let data: Value = serde_json::from_slice(payload)?;

        // Extract device_id - assume it's in the message
        let device_id = data
            .get("device_id")
            .or_else(|| data.get("deviceId"))
            .and_then(|v| v.as_str())
            .unwrap_or("default_device");

        // Extract settings fields
        let update = SettingUpdate {
            device_id: device_id.to_string(),
            indoor_target_temp: extract_f64(&data, &["indoor_target_temp", "indoorTargetTemp"]),
            mode: extract_i32(&data, &["mode"]),
            curve: extract_i32(&data, &["curve"]),
            curve_min: extract_i32(&data, &["curve_min", "curveMin"]),
            curve_max: extract_i32(&data, &["curve_max", "curveMax"]),
            curve_plus_5: extract_i32(&data, &["curve_plus_5", "curvePlus5"]),
            curve_zero: extract_i32(&data, &["curve_zero", "curveZero"]),
            curve_minus_5: extract_i32(&data, &["curve_minus_5", "curveMinus5"]),
            heatstop: extract_i32(&data, &["heatstop", "heatStop"]),
            integral_setting: extract_i16(&data, &["integral_setting", "integralSetting", "d73"]),
        };

        // Only upsert if we have at least one setting field
        if has_settings_data(&update) {
            self.repository.upsert(&update).await?;
            tracing::info!("Upserted settings for device: {}", device_id);
        } else {
            tracing::debug!("Message for device {} contains no settings data", device_id);
        }

        Ok(())
    }
}

/// Extract f64 value from JSON, trying multiple field names
fn extract_f64(data: &Value, field_names: &[&str]) -> Option<f64> {
    for field_name in field_names {
        if let Some(value) = data.get(field_name) {
            if let Some(num) = value.as_f64() {
                return Some(num);
            }
        }
    }
    None
}

/// Generic extraction function for numeric types from JSON
/// Tries multiple field names (for camelCase/snake_case compatibility)
/// Logs a warning if a value is found but out of range for the target type
fn extract_number<T>(data: &Value, field_names: &[&str]) -> Option<T>
where
    T: TryFrom<i64>,
{
    for field_name in field_names {
        if let Some(value) = data.get(field_name) {
            if let Some(num) = value.as_i64() {
                // Try to convert i64 to target type
                match T::try_from(num) {
                    Ok(converted) => return Some(converted),
                    Err(_) => {
                        tracing::warn!(
                            field = field_name,
                            value = num,
                            type_name = std::any::type_name::<T>(),
                            "Value out of range for type"
                        );
                        // Continue to next field name - maybe device uses different convention
                    }
                }
            }
        }
    }
    None
}

/// Extract i32 value from JSON, trying multiple field names
fn extract_i32(data: &Value, field_names: &[&str]) -> Option<i32> {
    extract_number::<i32>(data, field_names)
}

/// Extract i16 value from JSON, trying multiple field names
fn extract_i16(data: &Value, field_names: &[&str]) -> Option<i16> {
    extract_number::<i16>(data, field_names)
}

/// Check if update contains any settings data
fn has_settings_data(update: &SettingUpdate) -> bool {
    update.indoor_target_temp.is_some()
        || update.mode.is_some()
        || update.curve.is_some()
        || update.curve_min.is_some()
        || update.curve_max.is_some()
        || update.curve_plus_5.is_some()
        || update.curve_zero.is_some()
        || update.curve_minus_5.is_some()
        || update.heatstop.is_some()
        || update.integral_setting.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_f64() {
        let data = json!({
            "indoor_target_temp": 21.5,
            "indoorTargetTemp": 22.0
        });

        // Test snake_case extraction
        assert_eq!(extract_f64(&data, &["indoor_target_temp"]), Some(21.5));

        // Test camelCase extraction
        assert_eq!(extract_f64(&data, &["indoorTargetTemp"]), Some(22.0));

        // Test field alias priority (first match wins)
        assert_eq!(
            extract_f64(&data, &["indoor_target_temp", "indoorTargetTemp"]),
            Some(21.5)
        );

        // Test missing field
        assert_eq!(extract_f64(&data, &["missing_field"]), None);
    }

    #[test]
    fn test_extract_i32() {
        let data = json!({
            "mode": 1,
            "curve_max": 50
        });

        // Test extraction
        assert_eq!(extract_i32(&data, &["mode"]), Some(1));
        assert_eq!(extract_i32(&data, &["curve_max"]), Some(50));

        // Test missing field
        assert_eq!(extract_i32(&data, &["missing"]), None);

        // Test field aliases
        let data_camel = json!({"curveMax": 45});
        assert_eq!(
            extract_i32(&data_camel, &["curve_max", "curveMax"]),
            Some(45)
        );
    }

    #[test]
    fn test_extract_i16() {
        let data = json!({
            "integral_setting": 10,
            "integralSetting": 15,
            "d73": 20
        });

        // Test snake_case extraction
        assert_eq!(extract_i16(&data, &["integral_setting"]), Some(10));

        // Test camelCase extraction
        assert_eq!(extract_i16(&data, &["integralSetting"]), Some(15));

        // Test device parameter name (d73)
        assert_eq!(extract_i16(&data, &["d73"]), Some(20));

        // Test field alias priority (first match wins)
        assert_eq!(
            extract_i16(&data, &["integral_setting", "integralSetting", "d73"]),
            Some(10)
        );

        // Test missing field
        assert_eq!(extract_i16(&data, &["missing"]), None);

        // Test value range for i16
        let data_max = json!({"value": 32767}); // i16::MAX
        assert_eq!(extract_i16(&data_max, &["value"]), Some(32767));

        let data_min = json!({"value": -32768}); // i16::MIN
        assert_eq!(extract_i16(&data_min, &["value"]), Some(-32768));
    }

    #[test]
    fn test_extract_number_generic() {
        let data = json!({"value": 100});

        // Test generic extraction for different types
        assert_eq!(extract_number::<i16>(&data, &["value"]), Some(100i16));
        assert_eq!(extract_number::<i32>(&data, &["value"]), Some(100i32));
        assert_eq!(extract_number::<i64>(&data, &["value"]), Some(100i64));
    }

    #[test]
    fn test_extract_number_overflow() {
        // Test overflow handling for i16
        let data_overflow = json!({"value": 40000}); // > i16::MAX
        assert_eq!(extract_number::<i16>(&data_overflow, &["value"]), None);

        // i32 should handle this value
        assert_eq!(
            extract_number::<i32>(&data_overflow, &["value"]),
            Some(40000)
        );
    }

    #[test]
    fn test_has_settings_data() {
        // Test with no settings
        let empty = SettingUpdate {
            device_id: "test".to_string(),
            indoor_target_temp: None,
            mode: None,
            curve: None,
            curve_min: None,
            curve_max: None,
            curve_plus_5: None,
            curve_zero: None,
            curve_minus_5: None,
            heatstop: None,
            integral_setting: None,
        };
        assert!(!has_settings_data(&empty));

        // Test with integral_setting only
        let with_integral = SettingUpdate {
            device_id: "test".to_string(),
            indoor_target_temp: None,
            mode: None,
            curve: None,
            curve_min: None,
            curve_max: None,
            curve_plus_5: None,
            curve_zero: None,
            curve_minus_5: None,
            heatstop: None,
            integral_setting: Some(10),
        };
        assert!(has_settings_data(&with_integral));

        // Test with multiple settings
        let with_multiple = SettingUpdate {
            device_id: "test".to_string(),
            indoor_target_temp: Some(21.5),
            mode: None,
            curve: None,
            curve_min: None,
            curve_max: None,
            curve_plus_5: None,
            curve_zero: None,
            curve_minus_5: None,
            heatstop: Some(18),
            integral_setting: Some(10),
        };
        assert!(has_settings_data(&with_multiple));
    }
}
