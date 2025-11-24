use crate::config::Pipeline;
use crate::db::{insert_batch, DbPool};
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
pub struct RowEnvelope { pub table: String, pub row: Row }

impl Ingestor {
    pub fn new(pool: DbPool, batch_size: usize, linger_ms: u64) -> Self {
        let (tx, mut rx) = mpsc::channel::<RowEnvelope>(batch_size * 4);
        tokio::spawn(async move {
            let mut buffers: HashMap<String, Vec<Row>> = HashMap::new();
            let mut last_flush = Instant::now();
            loop {
                let timeout = tokio::time::sleep(Duration::from_millis(linger_ms));
                tokio::pin!(timeout);
                tokio::select! {
                    biased;
                    Some(env) = rx.recv() => {
                        let buf = buffers.entry(env.table.clone()).or_default();
                        buf.push(env.row);
                        if buf.len() >= batch_size {
                            let table = env.table;
                            if let Some(rows) = buffers.remove(&table) {
                                if let Err(e) = insert_batch(&pool, &table, &rows).await { error!(table=%table, "insert batch failed: {e}"); }
                                else { debug!(table=%table, count=rows.len(), "batch flushed (size)"); }
                            }
                        }
                    }
                    _ = &mut timeout => {
                        if last_flush.elapsed().as_millis() as u64 >= linger_ms {
                            let keys: Vec<String> = buffers.keys().cloned().collect();
                            for table in keys {
                                if let Some(rows) = buffers.remove(&table) { if !rows.is_empty() {
                                    if let Err(e) = insert_batch(&pool, &table, &rows).await { error!(table=%table, "insert batch failed: {e}"); }
                                    else { debug!(table=%table, count=rows.len(), "batch flushed (linger)"); }
                                }}
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

    pub async fn handle_message(&self, pipelines: &[Pipeline], topic: &str, payload: &[u8]) -> Result<(), AppError> {
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
                
                self.tx.send(RowEnvelope { table: p.table.clone(), row }).await.map_err(|e| AppError::Other(anyhow::anyhow!("send row: {}", e)))?;
            }
        }
        Ok(())
    }
    
    /// Check if enough time has passed since the last stored message for this pipeline
    fn should_store(&self, pipeline_name: &str, msg_time: &chrono::DateTime<chrono::Utc>, interval: &str) -> Result<bool, AppError> {
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

/// Parse interval string to chrono::Duration
fn parse_interval(interval: &str) -> Result<ChronoDuration, AppError> {
    match interval.to_uppercase().as_str() {
        "SECOND" => Ok(ChronoDuration::seconds(1)),
        "MINUTE" => Ok(ChronoDuration::minutes(1)),
        "HOUR" => Ok(ChronoDuration::hours(1)),
        "DAY" => Ok(ChronoDuration::days(1)),
        _ => Err(AppError::Other(anyhow::anyhow!("Unknown interval: {}. Supported: SECOND, MINUTE, HOUR, DAY", interval))),
    }
}
