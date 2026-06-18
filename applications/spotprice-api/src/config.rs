use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DbConfig,
    pub api: ApiConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    pub nordpool: NordpoolConfig,
    #[serde(default)]
    pub fetch: FetchConfig,
    #[serde(default)]
    pub apns: Option<ApnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_host")]
    pub host: String,
    #[serde(default = "default_api_port")]
    pub port: u16,
}

fn default_api_host() -> String {
    "0.0.0.0".into()
}

fn default_api_port() -> u16 {
    8080
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    // Multi-issuer JWKS configuration (for RS256 validation) plus optional
    // introspection for opaque tokens. Mirrors homelab-api's auth shape.
    #[serde(default)]
    pub issuers: Vec<IssuerConfig>,
    // Placeholder kept for config-file compatibility with the other services.
    #[serde(default)]
    pub jwt_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConfig {
    pub name: String,
    pub issuer: String,
    #[serde(default)]
    pub jwks_url: Option<String>,
    #[serde(default)]
    pub introspection_url: Option<String>,
    #[serde(default)]
    pub introspection_client_id: Option<String>,
    #[serde(default)]
    pub introspection_client_secret: Option<String>,
}

/// Nord Pool day-ahead price source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NordpoolConfig {
    #[serde(default = "default_nordpool_base_url")]
    pub base_url: String,
    #[serde(default = "default_delivery_area")]
    pub delivery_area: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    /// A browser-like User-Agent is required; the data portal API returns 403 without one.
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

fn default_nordpool_base_url() -> String {
    "https://dataportal-api.nordpoolgroup.com/api/DayAheadPrices".into()
}

fn default_delivery_area() -> String {
    "SE3".into()
}

fn default_currency() -> String {
    "SEK".into()
}

fn default_user_agent() -> String {
    "Mozilla/5.0 (compatible; homelab-spotprice/1.0)".into()
}

/// Daily fetch scheduling. Times are local (container TZ should be Europe/Stockholm).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchConfig {
    /// Earliest time of day to attempt fetching tomorrow's prices (HH:MM).
    #[serde(default = "default_window_start")]
    pub window_start: String,
    /// Latest time of day for the randomized target (HH:MM).
    #[serde(default = "default_window_end")]
    pub window_end: String,
    /// Keep retrying (if prices are not yet published) until this time (HH:MM).
    #[serde(default = "default_retry_until")]
    pub retry_until: String,
    /// Minimum seconds between fetch attempts during the retry window.
    #[serde(default = "default_retry_interval_secs")]
    pub retry_interval_secs: u64,
    /// How often the scheduler loop wakes up.
    #[serde(default = "default_check_interval_secs")]
    pub check_interval_secs: u64,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            window_start: default_window_start(),
            window_end: default_window_end(),
            retry_until: default_retry_until(),
            retry_interval_secs: default_retry_interval_secs(),
            check_interval_secs: default_check_interval_secs(),
        }
    }
}

fn default_window_start() -> String {
    "13:30".into()
}

fn default_window_end() -> String {
    "14:00".into()
}

fn default_retry_until() -> String {
    "16:00".into()
}

fn default_retry_interval_secs() -> u64 {
    300
}

fn default_check_interval_secs() -> u64 {
    60
}

/// Apple Push Notification service credentials (token-based, .p8 key).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApnsConfig {
    /// Path to the mounted PKCS#8 (.p8) key file.
    pub key_path: String,
    pub key_id: String,
    pub team_id: String,
    /// The app's bundle identifier (used as the APNs topic).
    pub bundle_id: String,
}

impl Config {
    /// Load YAML from disk, substitute $(VAR)/${VAR} with env vars, then parse.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let raw = std::fs::read_to_string(path)?;
        let expanded = expand_env_placeholders(&raw)?;
        let mut cfg: Self = serde_yaml::from_str(&expanded)?;

        // Optional: allow DATABASE_URL env to override whatever YAML had
        if let Ok(url) = std::env::var("DATABASE_URL") {
            cfg.database.url = url;
        }

        Ok(cfg)
    }
}

/// Expand $(VAR) and ${VAR} placeholders using environment variables.
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
                    it.next(); // consume '('
                    let var = read_until(&mut it, ')')
                        .context("unterminated env placeholder: missing ')'")?;
                    let val = std::env::var(&var)
                        .with_context(|| format!("missing environment variable: {}", var))?;
                    out.push_str(&val);
                }
                Some('{') => {
                    it.next(); // consume '{'
                    let var = read_until(&mut it, '}')
                        .context("unterminated env placeholder: missing '}'")?;
                    let val = std::env::var(&var)
                        .with_context(|| format!("missing environment variable: {}", var))?;
                    out.push_str(&val);
                }
                _ => {
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
