use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub redpanda: RedpandaConfig,
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive_secs: Option<u64>,
    pub clean_session: Option<bool>,
    /// "v5" or "v311"
    #[serde(default = "default_protocol")]
    pub protocol: String,
    pub tls: Option<TlsConfig>,
}

fn default_protocol() -> String {
    "v5".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to a PEM CA bundle (optional)
    pub ca_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedpandaConfig {
    /// Comma-separated list of broker addresses (e.g., "redpanda.redpanda.svc.cluster.local:9092")
    #[serde(default = "default_brokers")]
    pub brokers: String,
}

fn default_brokers() -> String {
    "redpanda.redpanda.svc.cluster.local:9092".into()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pipeline {
    pub name: String,
    pub topic: String,
    pub qos: u8,
    pub redpanda_topic: String,
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
    /// Afterwards, if REDPANDA_BROKERS env is set, override `redpanda.brokers`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let raw = fs::read_to_string(path)?;
        let expanded = expand_env_placeholders(&raw)?;
        let mut cfg: Self = serde_yaml::from_str(&expanded)?;

        // Optional: allow REDPANDA_BROKERS env to override whatever YAML had
        if let Ok(brokers) = std::env::var("REDPANDA_BROKERS") {
            cfg.redpanda.brokers = brokers;
        }

        anyhow::ensure!(
            !cfg.pipelines.is_empty(),
            "config must include at least one pipeline"
        );
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
    for ch in it.by_ref() {
        if ch == end {
            return Some(buf);
        }
        buf.push(ch);
    }
    None
}
