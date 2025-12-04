use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub redpanda: RedpandaConfig,
    pub database: DbConfig,
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedpandaConfig {
    pub brokers: String,
    /// Consumer group ID
    #[serde(default = "default_group_id")]
    pub group_id: String,
    /// Offset reset strategy: "earliest", "latest", or "none"
    #[serde(default = "default_auto_offset_reset")]
    pub auto_offset_reset: String,
}

fn default_group_id() -> String {
    "redpanda-sink".into()
}

fn default_auto_offset_reset() -> String {
    "earliest".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
    #[serde(default = "default_write")]
    pub write: WriteConfig,
}
fn default_write() -> WriteConfig {
    WriteConfig {
        batch_size: 500,
        linger_ms: 500,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteConfig {
    pub batch_size: usize,
    pub linger_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pipeline {
    pub name: String,
    pub topic: String,
    pub table: String,
    /// Data type: "timeseries" or "static"
    pub data_type: String,
    /// For static data: columns to use for upsert conflict resolution
    #[serde(default)]
    pub upsert_key: Option<Vec<String>>,
    pub timestamp: TimestampConfig,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    #[serde(default)]
    pub fields: BTreeMap<String, FieldConfig>,
    #[serde(default)]
    pub bit_flags: Option<Vec<BitFlagConfig>>,
    /// Optional interval for storing messages: "SECOND", "MINUTE", "HOUR", "DAY"
    /// If not set, all messages are stored
    pub store_interval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampConfig {
    /// JSONPath to the timestamp; if absent and use_now=true, now() is used.
    pub path: Option<String>,
    /// "rfc3339" | "unix_ms" | "unix_s" | "iso8601"
    #[serde(default = "default_ts_format")]
    pub format: String,
    #[serde(default = "default_use_now")]
    pub use_now: bool,
}
fn default_ts_format() -> String {
    "rfc3339".into()
}
fn default_use_now() -> bool {
    true
}
impl Default for TimestampConfig {
    fn default() -> Self {
        Self {
            path: None,
            format: default_ts_format(),
            use_now: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    pub path: String,
    /// "float" | "int" | "bool" | "text" | "nested"
    pub r#type: String,
    /// For nested type: map of attribute names to output column names
    #[serde(default)]
    pub attributes: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitFlagConfig {
    /// JSONPath to the byte value in the message
    pub source_path: String,
    /// Map of bit position (0-7) to output field name
    pub flags: BTreeMap<u8, String>,
}

impl Config {
    /// Load YAML from disk, substitute $(VAR)/${VAR} with env vars, then parse.
    /// Afterwards, if DATABASE_URL env is set, override `database.url`.
    /// If REDPANDA_BROKERS env is set, override `redpanda.brokers`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let raw = fs::read_to_string(path)?;
        let expanded = expand_env_placeholders(&raw)?;
        let mut cfg: Self = serde_yaml::from_str(&expanded)?;

        // Optional: allow DATABASE_URL env to override whatever YAML had
        if let Ok(url) = std::env::var("DATABASE_URL") {
            cfg.database.url = url;
        }

        // Optional: allow REDPANDA_BROKERS env to override whatever YAML had
        if let Ok(brokers) = std::env::var("REDPANDA_BROKERS") {
            cfg.redpanda.brokers = brokers;
        }

        anyhow::ensure!(
            !cfg.pipelines.is_empty(),
            "config must include at least one pipeline"
        );

        // Validate pipelines
        for pipeline in &cfg.pipelines {
            anyhow::ensure!(
                pipeline.data_type == "timeseries" || pipeline.data_type == "static",
                "pipeline '{}' data_type must be 'timeseries' or 'static'",
                pipeline.name
            );
            if pipeline.data_type == "static" {
                anyhow::ensure!(
                    pipeline.upsert_key.is_some()
                        && !pipeline.upsert_key.as_ref().unwrap().is_empty(),
                    "pipeline '{}' with data_type 'static' must specify upsert_key",
                    pipeline.name
                );
            }
        }

        Ok(cfg)
    }
}

/// Expand $(VAR) and ${VAR} placeholders using environment variables.
/// Notes:
/// - "$.something" (JSONPath) is NOT matched; only "$(" and "${" are.
/// - "$$" becomes a literal "$" (escape).
fn expand_env_placeholders(input: &str) -> Result<String, anyhow::Error> {
    use anyhow::Context;

    let mut out = String::with_capacity(input.len());
    let mut it = input.chars().peekable();

    while let Some(c) = it.next() {
        if c == '$' {
            match it.peek().copied() {
                Some('$') => {
                    // Escape "$$" -> "$"
                    it.next();
                    out.push('$');
                }
                Some('(') => {
                    // $(VAR)
                    it.next(); // consume '('
                    let var = read_until(&mut it, ')')
                        .context("unterminated env placeholder: missing ')'")?;
                    let val = std::env::var(&var)
                        .with_context(|| format!("missing environment variable: {}", var))?;
                    out.push_str(&val);
                }
                Some('{') => {
                    // ${VAR}
                    it.next(); // consume '{'
                    let var = read_until(&mut it, '}')
                        .context("unterminated env placeholder: missing '}'")?;
                    let val = std::env::var(&var)
                        .with_context(|| format!("missing environment variable: {}", var))?;
                    out.push_str(&val);
                }
                _ => {
                    // Not a placeholder; keep the '$' as-is
                    out.push('$');
                }
            }
        } else {
            out.push(c);
        }
    }

    Ok(out)
}

/// Read characters until we hit `end`, returning the collected string.
/// Consumes the closing delimiter.
fn read_until<I>(it: &mut std::iter::Peekable<I>, end: char) -> Option<String>
where
    I: Iterator<Item = char>,
{
    let mut buf = String::new();
    while let Some(ch) = it.next() {
        if ch == end {
            return Some(buf);
        }
        buf.push(ch);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config_str = r#"
redpanda:
  brokers: "localhost:9092"
  group_id: "test-group"

database:
  url: "postgres://localhost/test"

pipelines:
  - name: "test-timeseries"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags: {}
    fields: {}
  - name: "test-static"
    topic: "static-topic"
    table: "devices"
    data_type: "static"
    upsert_key: ["device_id"]
    timestamp:
      use_now: true
    tags: {}
    fields: {}
"#;

        let temp_file =
            std::env::temp_dir().join(format!("test-config-{}.yaml", std::process::id()));
        std::fs::write(&temp_file, config_str).unwrap();

        let config = Config::load(&temp_file).unwrap();
        assert_eq!(config.pipelines.len(), 2);
        assert_eq!(config.pipelines[0].data_type, "timeseries");
        assert_eq!(config.pipelines[1].data_type, "static");
        assert!(config.pipelines[1].upsert_key.is_some());

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_config_env_override() {
        let config_str = r#"
redpanda:
  brokers: "default:9092"

database:
  url: "postgres://default/test"

pipelines:
  - name: "test"
    topic: "test-topic"
    table: "telemetry"
    data_type: "timeseries"
    timestamp:
      use_now: true
    tags: {}
    fields: {}
"#;

        let temp_file =
            std::env::temp_dir().join(format!("test-config-env-{}.yaml", std::process::id()));
        std::fs::write(&temp_file, config_str).unwrap();

        let original_db = std::env::var("DATABASE_URL").ok();
        let original_brokers = std::env::var("REDPANDA_BROKERS").ok();

        std::env::set_var("DATABASE_URL", "postgres://override/test");
        std::env::set_var("REDPANDA_BROKERS", "override:9092");

        let config = Config::load(&temp_file).unwrap();
        assert_eq!(config.database.url, "postgres://override/test");
        assert_eq!(config.redpanda.brokers, "override:9092");

        if let Some(val) = original_db {
            std::env::set_var("DATABASE_URL", val);
        } else {
            std::env::remove_var("DATABASE_URL");
        }
        if let Some(val) = original_brokers {
            std::env::set_var("REDPANDA_BROKERS", val);
        } else {
            std::env::remove_var("REDPANDA_BROKERS");
        }

        std::fs::remove_file(&temp_file).ok();
    }
}
