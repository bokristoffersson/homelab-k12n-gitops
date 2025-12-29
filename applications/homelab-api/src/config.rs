use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DbConfig,
    pub api: ApiConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
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
    // Legacy HS256 configuration (for local auth)
    #[serde(default)]
    pub jwt_secret: Option<String>,
    #[serde(default = "default_jwt_expiry_hours")]
    pub jwt_expiry_hours: u64,
    #[serde(default)]
    pub users: Vec<User>,

    // JWKS configuration (for RS256 validation from Authentik)
    #[serde(default)]
    pub jwks_url: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
}

fn default_jwt_expiry_hours() -> u64 {
    24
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
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

        // Optional: allow JWT_SECRET env to override auth.jwt_secret
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            if let Some(ref mut auth) = cfg.auth {
                auth.jwt_secret = Some(jwt_secret);
            }
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
