use crate::config::Pipeline;
use crate::error::AppError;
use crate::mapping::{extract_row, topic_matches, FieldValue, Row};
use crate::redpanda::{publish_message, RedpandaProducer};
use chrono::Duration as ChronoDuration;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

pub struct Ingestor {
    producer: Arc<RedpandaProducer>,
    last_store_times: Arc<Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

impl Ingestor {
    pub fn new(producer: RedpandaProducer) -> Self {
        Self {
            producer: Arc::new(producer),
            last_store_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_message(
        &self,
        pipelines: &[Pipeline],
        topic: &str,
        payload: &[u8],
    ) -> Result<(), AppError> {
        for p in pipelines {
            if topic_matches(&p.topic, topic) {
                let row = extract_row(p, topic, payload)?;

                // Check if we should store based on time interval
                if let Some(interval) = &p.store_interval {
                    if !self.should_store(&p.name, &row.ts, interval)? {
                        debug!(
                            pipeline = %p.name,
                            topic = %topic,
                            ts = ?row.ts,
                            "skipping message due to interval filter"
                        );
                        continue;
                    }
                }

                // Transform Row to JSON for publishing
                let json_payload = row_to_json(&row)?;
                let payload_bytes = serde_json::to_vec(&json_payload)
                    .map_err(|e| AppError::Json(e))?;

                // Use pipeline name as key for partitioning (optional, can be None)
                let key = Some(p.name.as_str());

                // Publish to Redpanda
                if let Err(e) = publish_message(
                    &self.producer,
                    &p.redpanda_topic,
                    key,
                    &payload_bytes,
                )
                .await
                {
                    error!(
                        pipeline = %p.name,
                        redpanda_topic = %p.redpanda_topic,
                        error = %e,
                        "failed to publish message to Redpanda"
                    );
                    return Err(e);
                }

                debug!(
                    pipeline = %p.name,
                    redpanda_topic = %p.redpanda_topic,
                    "message published to Redpanda"
                );
            }
        }
        Ok(())
    }

    /// Check if enough time has passed since the last stored message for this pipeline
    /// This is public for testing purposes
    pub fn should_store(
        &self,
        pipeline_name: &str,
        msg_time: &chrono::DateTime<chrono::Utc>,
        interval: &str,
    ) -> Result<bool, AppError> {
        let duration = parse_interval(interval)?;
        let mut times = self.last_store_times.lock().unwrap();

        if let Some(last_time) = times.get(pipeline_name) {
            let elapsed = *msg_time - *last_time;
            if elapsed < duration {
                return Ok(false);
            }
        }

        // Update the last store time for this pipeline
        times.insert(pipeline_name.to_string(), *msg_time);
        Ok(true)
    }
}

/// Convert Row to JSON format for publishing to Redpanda
fn row_to_json(row: &Row) -> Result<Value, AppError> {
    let mut obj = Map::new();

    // Add timestamp
    obj.insert(
        "ts".to_string(),
        Value::String(row.ts.to_rfc3339()),
    );

    // Add tags
    if !row.tags.is_empty() {
        let mut tags_obj = Map::new();
        for (k, v) in &row.tags {
            tags_obj.insert(k.clone(), Value::String(v.clone()));
        }
        obj.insert("tags".to_string(), Value::Object(tags_obj));
    }

    // Add fields
    if !row.fields.is_empty() {
        let mut fields_obj = Map::new();
        for (k, v) in &row.fields {
            let field_value = match v {
                FieldValue::F64(f) => Value::Number(
                    serde_json::Number::from_f64(*f)
                        .ok_or_else(|| AppError::Other(anyhow::anyhow!("Invalid f64: {}", f)))?,
                ),
                FieldValue::I64(i) => Value::Number((*i).into()),
                FieldValue::Bool(b) => Value::Bool(*b),
                FieldValue::Text(t) => Value::String(t.clone()),
            };
            fields_obj.insert(k.clone(), field_value);
        }
        obj.insert("fields".to_string(), Value::Object(fields_obj));
    }

    Ok(Value::Object(obj))
}

/// Parse interval string to chrono::Duration
fn parse_interval(interval: &str) -> Result<ChronoDuration, AppError> {
    match interval.to_uppercase().as_str() {
        "SECOND" => Ok(ChronoDuration::seconds(1)),
        "MINUTE" => Ok(ChronoDuration::minutes(1)),
        "HOUR" => Ok(ChronoDuration::hours(1)),
        "DAY" => Ok(ChronoDuration::days(1)),
        _ => Err(AppError::Other(anyhow::anyhow!(
            "Unknown interval: {}. Supported: SECOND, MINUTE, HOUR, DAY",
            interval
        ))),
    }
}
