//! Multi-issuer JWT validation via JWKS (RS256), pure-Rust (jsonwebtoken/ring).
//!
//! The mobile app sends Authelia JWT access tokens (RFC 9068) as Bearer tokens.
//! We validate the signature against the issuer's JWKS and check `iss`/`exp`.
//! No opaque-token introspection and no system OpenSSL dependency.

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: String,        // subject (username / client id)
    pub exp: usize,         // expiration time
    pub iat: Option<usize>, // issued at (optional)
    pub iss: Option<String>,
    pub email: Option<String>,
    // RFC 8693 `scope`: space-separated string. Array form tolerated.
    // Authelia JWT access tokens (RFC 9068) put scopes in `scp` (array).
    #[serde(default, alias = "scp", deserialize_with = "deserialize_scope")]
    pub scope: Vec<String>,
}

impl Claims {
    #[allow(dead_code)]
    pub fn has_scope(&self, required: &str) -> bool {
        self.scope.iter().any(|s| s == required)
    }

    pub fn all_scopes(&self) -> Vec<String> {
        self.scope.clone()
    }
}

fn deserialize_scope<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(parse_scope_value(value.as_ref()))
}

fn parse_scope_value(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(s)) => s.split_whitespace().map(|part| part.to_string()).collect(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// Reasons a token can be rejected. The middleware maps all of these to 401.
#[derive(Debug)]
pub enum JwtError {
    /// No issuer accepted the token (bad signature, wrong issuer, expired, ...).
    Invalid,
}

/// One configured issuer with its fetched JWKS.
#[derive(Clone)]
struct IssuerEntry {
    name: String,
    issuer: String,
    jwks: Arc<RwLock<JwkSet>>,
}

/// Multi-issuer JWKS token validator.
#[derive(Clone)]
pub struct JwtValidator {
    issuers: Vec<IssuerEntry>,
}

impl JwtValidator {
    /// Build a validator from the configured issuers. Each issuer must provide a
    /// `jwks_url`; its key set is fetched once at startup.
    pub async fn new_multi(
        configs: Vec<crate::config::IssuerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut issuers = Vec::new();

        for config in configs {
            let url = config
                .jwks_url
                .clone()
                .ok_or_else(|| format!("issuer '{}' is missing required jwks_url", config.name))?;
            let jwks = fetch_jwks(&url).await?;
            info!("Loaded JWKS for issuer '{}' from {}", config.name, url);
            issuers.push(IssuerEntry {
                name: config.name,
                issuer: config.issuer,
                jwks: Arc::new(RwLock::new(jwks)),
            });
        }

        Ok(Self { issuers })
    }

    pub fn issuer_count(&self) -> usize {
        self.issuers.len()
    }

    /// Validate a Bearer token against every configured issuer. Returns the
    /// decoded claims on the first issuer that accepts it.
    pub async fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        for entry in &self.issuers {
            match self.validate_with_issuer(token, entry).await {
                Ok(claims) => {
                    debug!("JWT validated by issuer '{}'", entry.name);
                    return Ok(claims);
                }
                Err(e) => debug!("issuer '{}' rejected token: {:?}", entry.name, e),
            }
        }
        warn!("Token rejected by all configured issuers");
        Err(JwtError::Invalid)
    }

    async fn validate_with_issuer(
        &self,
        token: &str,
        entry: &IssuerEntry,
    ) -> Result<Claims, JwtError> {
        let header = decode_header(token).map_err(|_| JwtError::Invalid)?;
        let kid = header.kid.ok_or(JwtError::Invalid)?;

        let jwks = entry.jwks.read().await;
        let jwk = jwks.find(&kid).ok_or(JwtError::Invalid)?;
        let key = DecodingKey::from_jwk(jwk).map_err(|_| JwtError::Invalid)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[entry.issuer.as_str()]);
        validation.set_required_spec_claims(&["exp", "sub"]);
        // This is a resource server: it authorizes on scope, not audience.
        // Authelia access tokens carry an `aud` claim, and jsonwebtoken validates
        // audience by default — without an expected audience set that rejects the
        // token outright. Validate signature + issuer + exp only (matching the
        // other homelab APIs).
        validation.validate_aud = false;

        decode::<Claims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                debug!("token rejected by issuer '{}': {}", entry.name, e);
                JwtError::Invalid
            })
    }
}

async fn fetch_jwks(url: &str) -> Result<JwkSet, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get(url).await?;
    let jwks: JwkSet = res.json().await?;
    Ok(jwks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scope_value_handles_string() {
        let value = serde_json::json!("read:spotprice read:energy");
        let scopes = parse_scope_value(Some(&value));
        assert_eq!(scopes, vec!["read:spotprice", "read:energy"]);
    }

    #[test]
    fn parse_scope_value_handles_array() {
        let value = serde_json::json!(["read:spotprice", "write:plugs"]);
        let scopes = parse_scope_value(Some(&value));
        assert_eq!(scopes, vec!["read:spotprice", "write:plugs"]);
    }

    #[test]
    fn parse_scope_value_handles_missing_and_empty() {
        assert!(parse_scope_value(None).is_empty());
        assert!(parse_scope_value(Some(&serde_json::json!(""))).is_empty());
        assert!(parse_scope_value(Some(&serde_json::json!(null))).is_empty());
    }

    #[test]
    fn claims_deserializes_scope_string() {
        let raw = serde_json::json!({
            "sub": "alice",
            "exp": 9_999_999_999_usize,
            "scope": "read:spotprice read:energy"
        });
        let claims: Claims = serde_json::from_value(raw).unwrap();
        assert!(claims.has_scope("read:spotprice"));
        assert!(claims.has_scope("read:energy"));
        assert!(!claims.has_scope("write:plugs"));
    }

    #[test]
    fn claims_deserializes_scp_array() {
        // Authelia JWT access tokens use `scp` as an array.
        let raw = serde_json::json!({
            "sub": "alice",
            "exp": 9_999_999_999_usize,
            "scp": ["read:spotprice"]
        });
        let claims: Claims = serde_json::from_value(raw).unwrap();
        assert!(claims.has_scope("read:spotprice"));
        assert!(!claims.has_scope("read:energy"));
    }
}
