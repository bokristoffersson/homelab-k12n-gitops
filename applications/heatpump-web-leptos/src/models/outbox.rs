use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Outbox entry for async command processing
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub id: i64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: HashMap<String, serde_json::Value>,
    pub status: OutboxStatus,
    pub created_at: String,
    pub published_at: Option<String>,
    pub confirmed_at: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
}

/// Outbox status enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutboxStatus {
    Pending,
    Published,
    Confirmed,
    Failed,
}

#[allow(dead_code)]
impl OutboxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Published => "published",
            Self::Confirmed => "confirmed",
            Self::Failed => "failed",
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Confirmed | Self::Failed)
    }
}

/// Outbox list response
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxResponse {
    pub data: Vec<OutboxEntry>,
}
