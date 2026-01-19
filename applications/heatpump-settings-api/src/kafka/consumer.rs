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

/// Extract i32 value from JSON, trying multiple field names
fn extract_i32(data: &Value, field_names: &[&str]) -> Option<i32> {
    for field_name in field_names {
        if let Some(value) = data.get(field_name) {
            if let Some(num) = value.as_i64() {
                return Some(num as i32);
            }
        }
    }
    None
}

fn extract_i16(data: &Value, field_names: &[&str]) -> Option<i16> {
    for field_name in field_names {
        if let Some(value) = data.get(field_name) {
            if let Some(num) = value.as_i64() {
                return Some(num as i16);
            }
        }
    }
    None
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
}
