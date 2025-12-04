use crate::config::Pipeline;
use crate::db::{insert_batch, upsert_batch, DbPool};
use crate::error::AppError;
use crate::mapping::{extract_row, topic_matches, Row};
use chrono::Duration as ChronoDuration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, error};

pub struct Ingestor {
    tx: mpsc::Sender<RowEnvelope>,
    last_store_times: Arc<Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

#[derive(Debug)]
pub struct RowEnvelope {
    pub table: String,
    pub row: Row,
    pub data_type: String,
    pub upsert_key: Option<Vec<String>>,
}

impl Ingestor {
    pub fn new(pool: DbPool, batch_size: usize, linger_ms: u64) -> Self {
        let (tx, mut rx) = mpsc::channel::<RowEnvelope>(batch_size * 4);
        tokio::spawn(async move {
            let mut buffers: HashMap<String, Vec<RowEnvelope>> = HashMap::new();
            let mut last_flush = Instant::now();
            loop {
                let timeout = tokio::time::sleep(Duration::from_millis(linger_ms));
                tokio::pin!(timeout);
                tokio::select! {
                    biased;
                    Some(env) = rx.recv() => {
                        let key = format!("{}:{}", env.table, env.data_type);
                        let buf = buffers.entry(key.clone()).or_default();
                        buf.push(env);
                        if buf.len() >= batch_size {
                            if let Some(rows_env) = buffers.remove(&key) {
                                if let Err(e) = flush_batch(&pool, &rows_env).await {
                                    error!(table=%rows_env[0].table, data_type=%rows_env[0].data_type, "batch flush failed: {e}");
                                } else {
                                    debug!(table=%rows_env[0].table, data_type=%rows_env[0].data_type, count=rows_env.len(), "batch flushed (size)");
                                }
                            }
                        }
                    }
                    _ = &mut timeout => {
                        if last_flush.elapsed().as_millis() as u64 >= linger_ms {
                            let keys: Vec<String> = buffers.keys().cloned().collect();
                            for key in keys {
                                if let Some(rows_env) = buffers.remove(&key) {
                                    if !rows_env.is_empty() {
                                        if let Err(e) = flush_batch(&pool, &rows_env).await {
                                            error!(table=%rows_env[0].table, data_type=%rows_env[0].data_type, "batch flush failed: {e}");
                                        } else {
                                            debug!(table=%rows_env[0].table, data_type=%rows_env[0].data_type, count=rows_env.len(), "batch flushed (linger)");
                                        }
                                    }
                                }
                            }
                            last_flush = Instant::now();
                        }
                    }
                }
            }
        });
        Self {
            tx,
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
                        debug!(pipeline=%p.name, topic=%topic, ts=?row.ts, "skipping message due to interval filter");
                        continue;
                    }
                }

                self.tx
                    .send(RowEnvelope {
                        table: p.table.clone(),
                        row,
                        data_type: p.data_type.clone(),
                        upsert_key: p.upsert_key.clone(),
                    })
                    .await
                    .map_err(|e| AppError::Other(anyhow::anyhow!("send row: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Check if enough time has passed since the last stored message for this pipeline
    fn should_store(
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

async fn flush_batch(pool: &DbPool, rows_env: &[RowEnvelope]) -> Result<(), AppError> {
    if rows_env.is_empty() {
        return Ok(());
    }

    let data_type = &rows_env[0].data_type;
    let table = &rows_env[0].table;
    let rows: Vec<Row> = rows_env.iter().map(|e| e.row.clone()).collect();

    match data_type.as_str() {
        "timeseries" => insert_batch(pool, table, &rows).await,
        "static" => {
            let upsert_key = rows_env[0]
                .upsert_key
                .as_ref()
                .ok_or_else(|| AppError::Config("upsert_key missing for static data".into()))?;
            upsert_batch(pool, table, upsert_key, &rows).await
        }
        other => Err(AppError::Config(format!("unknown data_type: {}", other))),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval() {
        assert!(parse_interval("SECOND").is_ok());
        assert!(parse_interval("MINUTE").is_ok());
        assert!(parse_interval("HOUR").is_ok());
        assert!(parse_interval("DAY").is_ok());
        assert!(parse_interval("invalid").is_err());
    }

    #[test]
    fn test_interval_durations() {
        let second = parse_interval("SECOND").unwrap();
        let minute = parse_interval("MINUTE").unwrap();
        assert_eq!(minute.num_seconds(), 60);
        assert_eq!(second.num_seconds(), 1);
    }
}
