# BPM Implementation Starter Guide

This guide provides a practical starting point for implementing your BPM system, building on your existing infrastructure.

---

## Quick Start: Minimal Process Engine

### Step 1: Database Schema

Create the process state tables in TimescaleDB:

```sql
-- Process definitions
CREATE TABLE IF NOT EXISTS process_definitions (
    id TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    name TEXT NOT NULL,
    definition JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, version)
);

-- Process instances
CREATE TABLE IF NOT EXISTS process_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id TEXT NOT NULL,
    definition_version INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'suspended')),
    variables JSONB NOT NULL DEFAULT '{}',
    current_activity TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    FOREIGN KEY (definition_id, definition_version) 
        REFERENCES process_definitions(id, version)
);

CREATE INDEX idx_process_instances_status ON process_instances(status);
CREATE INDEX idx_process_instances_definition ON process_instances(definition_id, definition_version);

-- Execution history (audit log)
CREATE TABLE IF NOT EXISTS process_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id UUID NOT NULL REFERENCES process_instances(id) ON DELETE CASCADE,
    activity_id TEXT,
    event_type TEXT NOT NULL, -- 'started', 'completed', 'failed', 'variable_changed'
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    data JSONB,
    message TEXT
);

CREATE INDEX idx_process_history_instance ON process_history(instance_id);
CREATE INDEX idx_process_history_timestamp ON process_history(timestamp);

-- Convert history to hypertable for time-series queries
SELECT create_hypertable('process_history', 'timestamp', if_not_exists => TRUE);
```

---

### Step 2: Simple Process Definition Format

Start with a simple YAML format:

```yaml
# Example: temperature-control.yaml
id: temperature-control
name: Temperature Control Process
version: 1

trigger:
  type: redpanda
  topic: heatpump-telemetry
  condition: |
    fields.flow_temp_c > 25.0

variables:
  - name: device_id
    source: tags.device_id
  - name: temperature
    source: fields.flow_temp_c
  - name: threshold
    value: 25.0

activities:
  - id: start
    type: start
    
  - id: check_temp
    type: decision
    condition: ${temperature} > ${threshold}
    true_path: activate_cooling
    false_path: log_normal
    
  - id: activate_cooling
    type: service
    action: mqtt_publish
    config:
      topic: home/heatpump/control
      payload:
        device_id: ${device_id}
        command: activate_cooling
    next: wait_recheck
    
  - id: wait_recheck
    type: timer
    duration: 5m
    next: recheck_temp
    
  - id: recheck_temp
    type: decision
    condition: ${temperature} <= ${threshold}
    true_path: log_success
    false_path: escalate
    
  - id: log_normal
    type: service
    action: log
    config:
      message: "Temperature normal: ${temperature}°C"
    next: end
    
  - id: log_success
    type: service
    action: log
    config:
      message: "Cooling successful"
    next: end
    
  - id: escalate
    type: service
    action: notify
    config:
      channel: alert
      message: "Temperature still high after cooling"
    next: end
    
  - id: end
    type: end
```

---

### Step 3: Basic Process Engine Structure (Rust)

Create a new service: `applications/process-engine/`

**Cargo.toml**:
```toml
[package]
name = "process-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
rdkafka = { version = "0.36", features = ["cmake-build"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

**src/main.rs** (skeleton):
```rust
mod config;
mod db;
mod engine;
mod process;
mod trigger;

use config::Config;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    let cfg = Config::load()?;
    info!("Process engine starting");

    // Connect to database
    let pool = db::connect(&cfg.database.url).await?;
    info!("Connected to database");

    // Connect to Redpanda
    let consumer = trigger::create_consumer(&cfg.redpanda.brokers).await?;
    info!("Connected to Redpanda");

    // Load process definitions
    let definitions = process::load_definitions(&pool).await?;
    info!("Loaded {} process definitions", definitions.len());

    // Start process engine
    let engine = engine::ProcessEngine::new(pool.clone());
    
    // Start trigger listener
    trigger::start_listener(consumer, definitions, engine).await?;

    Ok(())
}
```

**src/config.rs**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redpanda: RedpandaConfig,
    pub process_definitions_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedpandaConfig {
    pub brokers: String,
    pub group_id: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        // Load from environment or config file
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let redpanda_brokers = std::env::var("REDPANDA_BROKERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());
        let group_id = std::env::var("PROCESS_ENGINE_GROUP_ID")
            .unwrap_or_else(|_| "process-engine".to_string());

        Ok(Config {
            database: DatabaseConfig { url: database_url },
            redpanda: RedpandaConfig {
                brokers: redpanda_brokers,
                group_id,
            },
            process_definitions_path: std::env::var("PROCESS_DEFINITIONS_PATH")
                .unwrap_or_else(|_| "processes/".to_string()),
        })
    }
}
```

**src/process.rs** (process definition):
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDefinition {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub trigger: Trigger,
    pub variables: Vec<Variable>,
    pub activities: Vec<Activity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub r#type: String, // "redpanda", "database", "api", "timer"
    pub topic: Option<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub source: Option<String>, // JSONPath to extract from event
    pub value: Option<serde_json::Value>, // Static value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub r#type: String, // "start", "end", "decision", "service", "timer"
    pub condition: Option<String>,
    pub true_path: Option<String>,
    pub false_path: Option<String>,
    pub next: Option<String>,
    pub action: Option<String>,
    pub config: Option<serde_json::Value>,
    pub duration: Option<String>, // e.g., "5m", "1h"
}

impl ProcessDefinition {
    pub fn load_from_yaml(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let def: ProcessDefinition = serde_yaml::from_str(&content)?;
        Ok(def)
    }
}
```

**src/engine.rs** (process execution):
```rust
use crate::process::ProcessDefinition;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ProcessEngine {
    pool: PgPool,
}

impl ProcessEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_instance(
        &self,
        definition: &ProcessDefinition,
        initial_variables: HashMap<String, serde_json::Value>,
    ) -> Result<Uuid, anyhow::Error> {
        let instance_id = Uuid::new_v4();

        // Create process instance
        sqlx::query(
            r#"
            INSERT INTO process_instances 
                (id, definition_id, definition_version, status, variables, current_activity)
            VALUES ($1, $2, $3, 'running', $4, 'start')
            "#,
        )
        .bind(instance_id)
        .bind(&definition.id)
        .bind(definition.version as i32)
        .bind(serde_json::to_value(initial_variables)?)
        .execute(&self.pool)
        .await?;

        // Log start event
        self.log_event(&instance_id, "start", "started", None, None)
            .await?;

        // Start execution
        self.execute_next(&instance_id, definition, "start").await?;

        Ok(instance_id)
    }

    async fn execute_next(
        &self,
        instance_id: &Uuid,
        definition: &ProcessDefinition,
        activity_id: &str,
    ) -> Result<(), anyhow::Error> {
        let activity = definition
            .activities
            .iter()
            .find(|a| a.id == activity_id)
            .ok_or_else(|| anyhow::anyhow!("Activity not found: {}", activity_id))?;

        match activity.r#type.as_str() {
            "start" => {
                // Move to first activity after start
                if let Some(next) = &activity.next {
                    self.execute_next(instance_id, definition, next).await?;
                }
            }
            "decision" => {
                // Evaluate condition and route
                let condition_result = self.evaluate_condition(instance_id, &activity.condition)?;
                let next_activity = if condition_result {
                    activity.true_path.as_ref()
                } else {
                    activity.false_path.as_ref()
                };

                if let Some(next) = next_activity {
                    self.execute_next(instance_id, definition, next).await?;
                }
            }
            "service" => {
                // Execute service action
                self.execute_service(instance_id, activity).await?;
                if let Some(next) = &activity.next {
                    self.execute_next(instance_id, definition, next).await?;
                }
            }
            "timer" => {
                // Schedule timer (simplified: just log for now)
                self.log_event(instance_id, activity_id, "timer_started", None, None)
                    .await?;
                // TODO: Implement actual timer scheduling
                if let Some(next) = &activity.next {
                    self.execute_next(instance_id, definition, next).await?;
                }
            }
            "end" => {
                // Complete process
                sqlx::query(
                    "UPDATE process_instances SET status = 'completed', completed_at = NOW() WHERE id = $1",
                )
                .bind(instance_id)
                .execute(&self.pool)
                .await?;
                self.log_event(instance_id, "end", "completed", None, None)
                    .await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown activity type: {}", activity.r#type));
            }
        }

        Ok(())
    }

    fn evaluate_condition(
        &self,
        _instance_id: &Uuid,
        condition: &Option<String>,
    ) -> Result<bool, anyhow::Error> {
        // TODO: Implement condition evaluation with variable substitution
        // For now, simple placeholder
        if let Some(cond) = condition {
            // This is a simplified version - you'd need a proper expression evaluator
            Ok(true) // Placeholder
        } else {
            Ok(true)
        }
    }

    async fn execute_service(
        &self,
        instance_id: &Uuid,
        activity: &crate::process::Activity,
    ) -> Result<(), anyhow::Error> {
        if let Some(action) = &activity.action {
            match action.as_str() {
                "mqtt_publish" => {
                    // TODO: Publish to MQTT
                    tracing::info!("MQTT publish action (not implemented yet)");
                }
                "log" => {
                    // TODO: Log message
                    tracing::info!("Log action (not implemented yet)");
                }
                "notify" => {
                    // TODO: Send notification
                    tracing::info!("Notify action (not implemented yet)");
                }
                _ => {
                    return Err(anyhow::anyhow!("Unknown action: {}", action));
                }
            }
        }

        self.log_event(instance_id, &activity.id, "completed", None, None)
            .await?;

        Ok(())
    }

    async fn log_event(
        &self,
        instance_id: &Uuid,
        activity_id: &str,
        event_type: &str,
        data: Option<serde_json::Value>,
        message: Option<String>,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            INSERT INTO process_history 
                (instance_id, activity_id, event_type, data, message)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(instance_id)
        .bind(activity_id)
        .bind(event_type)
        .bind(data)
        .bind(message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

---

### Step 4: Event Trigger Listener

**src/trigger.rs**:
```rust
use crate::engine::ProcessEngine;
use crate::process::ProcessDefinition;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use std::collections::HashMap;
use tracing::{info, error};

pub async fn create_consumer(brokers: &str) -> Result<StreamConsumer, anyhow::Error> {
    let consumer: StreamConsumer = rdkafka::config::ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", "process-engine")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()?;

    Ok(consumer)
}

pub async fn start_listener(
    consumer: StreamConsumer,
    definitions: Vec<ProcessDefinition>,
    engine: ProcessEngine,
) -> Result<(), anyhow::Error> {
    // Subscribe to topics from all definitions
    let topics: Vec<&str> = definitions
        .iter()
        .filter_map(|d| d.trigger.topic.as_deref())
        .collect();

    consumer.subscribe(&topics.iter().map(|s| *s).collect::<Vec<_>>())?;
    info!("Subscribed to topics: {:?}", topics);

    // Start consuming messages
    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(payload) {
                        // Check each definition for matching trigger
                        for definition in &definitions {
                            if should_trigger(&definition.trigger, &json) {
                                // Extract variables
                                let variables = extract_variables(&definition.variables, &json);
                                
                                // Start process instance
                                match engine.start_instance(definition, variables).await {
                                    Ok(instance_id) => {
                                        info!(
                                            process = %definition.id,
                                            instance_id = %instance_id,
                                            "Started process instance"
                                        );
                                    }
                                    Err(e) => {
                                        error!(
                                            process = %definition.id,
                                            error = %e,
                                            "Failed to start process instance"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Error receiving message");
            }
        }
    }
}

fn should_trigger(trigger: &crate::process::Trigger, data: &serde_json::Value) -> bool {
    // TODO: Implement condition evaluation
    // For now, simple check
    if let Some(condition) = &trigger.condition {
        // Evaluate condition against data
        // This is simplified - you'd need a proper expression evaluator
        true // Placeholder
    } else {
        true
    }
}

fn extract_variables(
    variables: &[crate::process::Variable],
    data: &serde_json::Value,
) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();

    for var in variables {
        if let Some(source) = &var.source {
            // TODO: Implement JSONPath extraction
            // For now, simple field access
            if let Some(value) = data.get(source.trim_start_matches("$.")) {
                result.insert(var.name.clone(), value.clone());
            }
        } else if let Some(value) = &var.value {
            result.insert(var.name.clone(), value.clone());
        }
    }

    result
}
```

---

### Step 5: Database Module

**src/db.rs**:
```rust
use sqlx::PgPool;

pub async fn connect(url: &str) -> Result<PgPool, anyhow::Error> {
    let pool = PgPool::connect(url).await?;
    Ok(pool)
}
```

---

## Next Steps

1. **Implement the skeleton** above
2. **Add condition evaluation** (use a library like `evalexpr` or `rhai`)
3. **Implement service actions** (MQTT publish, notifications, etc.)
4. **Add timer support** (use `tokio::time` or a scheduler)
5. **Add error handling and retries**
6. **Add process state persistence** (save state after each activity)
7. **Add REST API** for manual process management

---

## Testing

Create a simple test process definition and trigger it:

```bash
# Start the process engine
DATABASE_URL=postgres://... REDPANDA_BROKERS=localhost:9092 cargo run

# Publish a test event to Redpanda
# This should trigger your temperature control process
```

---

## Resources

- **Expression Evaluation**: Consider `evalexpr` or `rhai` for condition evaluation
- **JSONPath**: Use `jsonpath_lib` for variable extraction
- **Timer Scheduling**: Use `tokio::time` for delays, or `cron` for scheduled tasks
- **MQTT Client**: Use `rumqttc` (same as your mqtt-input service)

---

*This is a minimal starting point. Expand it incrementally based on your needs!*
