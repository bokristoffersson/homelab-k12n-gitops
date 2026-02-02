use crate::error::{AppError, Result};
use alcoholic_jwt::{validate, Validation as JwksValidation, ValidationError, JWKS};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject (username)
    pub exp: usize,            // Expiration time
    pub iat: Option<usize>,    // Issued at (optional)
    pub iss: Option<String>,   // Issuer (for JWKS validation)
    pub email: Option<String>, // Email (from Authentik)
}

// Single issuer entry with cached JWKS
#[derive(Clone)]
struct IssuerEntry {
    name: String,
    issuer: String,
    jwks: Arc<RwLock<JWKS>>,
}

// Multi-issuer JWKS-based JWT Validator for RS256 tokens from Authentik
#[derive(Clone)]
pub struct JwtValidator {
    issuers: Vec<IssuerEntry>,
    issuer_index: HashMap<String, usize>,
}

impl JwtValidator {
    /// Create a new multi-issuer validator
    pub async fn new_multi(
        configs: Vec<crate::config::IssuerConfig>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut issuers = Vec::new();
        let mut issuer_index = HashMap::new();

        for (idx, config) in configs.into_iter().enumerate() {
            let jwks = fetch_jwks(&config.jwks_url).await?;
            info!(
                "Loaded JWKS for issuer '{}' from {}",
                config.name, config.jwks_url
            );

            issuers.push(IssuerEntry {
                name: config.name.clone(),
                issuer: config.issuer.clone(),
                jwks: Arc::new(RwLock::new(jwks)),
            });
            issuer_index.insert(config.issuer, idx);
        }

        Ok(Self {
            issuers,
            issuer_index,
        })
    }

    /// Legacy constructor for single issuer (backwards compatibility)
    pub async fn new(
        jwks_url: &str,
        issuer: String,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let jwks = fetch_jwks(jwks_url).await?;
        let mut issuer_index = HashMap::new();
        issuer_index.insert(issuer.clone(), 0);

        Ok(Self {
            issuers: vec![IssuerEntry {
                name: "default".to_string(),
                issuer,
                jwks: Arc::new(RwLock::new(jwks)),
            }],
            issuer_index,
        })
    }

    pub async fn validate_token(
        &self,
        token: &str,
    ) -> std::result::Result<Claims, ValidationError> {
        // First, try to extract the issuer from the token to find the right JWKS
        if let Some(iss) = extract_issuer_from_token(token) {
            if let Some(&idx) = self.issuer_index.get(&iss) {
                let entry = &self.issuers[idx];
                debug!("Validating JWT from issuer '{}' ({})", entry.name, iss);
                return self.validate_with_issuer(token, entry).await;
            } else {
                debug!("Unknown issuer in token: {}", iss);
            }
        }

        // If issuer extraction failed or issuer not found, try all issuers
        debug!(
            "Trying all {} configured issuers for JWT validation",
            self.issuers.len()
        );
        for entry in &self.issuers {
            match self.validate_with_issuer(token, entry).await {
                Ok(claims) => {
                    debug!("JWT validated successfully with issuer '{}'", entry.name);
                    return Ok(claims);
                }
                Err(e) => {
                    debug!(
                        "JWT validation failed with issuer '{}': {:?}",
                        entry.name, e
                    );
                }
            }
        }

        warn!("JWT validation failed with all configured issuers");
        Err(ValidationError::InvalidSignature)
    }

    async fn validate_with_issuer(
        &self,
        token: &str,
        entry: &IssuerEntry,
    ) -> std::result::Result<Claims, ValidationError> {
        let jwks = entry.jwks.read().await;
        let validations = vec![
            JwksValidation::Issuer(entry.issuer.clone()),
            JwksValidation::SubjectPresent,
        ];

        let kid = alcoholic_jwt::token_kid(token)
            .map_err(|_| ValidationError::InvalidSignature)?
            .ok_or(ValidationError::InvalidSignature)?;

        let jwk = jwks.find(&kid).ok_or(ValidationError::InvalidSignature)?;

        let valid_jwt = validate(token, jwk, validations)?;

        serde_json::from_value(valid_jwt.claims).map_err(|_| ValidationError::InvalidSignature)
    }

    #[allow(dead_code)]
    pub fn issuer_count(&self) -> usize {
        self.issuers.len()
    }
}

/// Extract issuer from JWT without validating signature
fn extract_issuer_from_token(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode the payload (second part)
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    claims.get("iss").and_then(|v| v.as_str()).map(String::from)
}

async fn fetch_jwks(
    url: &str,
) -> std::result::Result<JWKS, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get(url).await?;
    let jwks: JWKS = res.json().await?;
    Ok(jwks)
}

/// Legacy HS256 token validation (for backward compatibility)
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let validation = Validation::default();
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn create_test_token(username: &str, secret: &str, exp_offset_secs: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: username.to_string(),
            exp: (now as i64 + exp_offset_secs) as usize,
            iat: Some(now),
            iss: None,
            email: None,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn test_validate_valid_token() {
        let secret = "test-secret";
        let token = create_test_token("testuser", secret, 3600); // 1 hour from now

        let result = validate_token(&token, secret);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, "testuser");
    }

    #[test]
    fn test_validate_expired_token() {
        let secret = "test-secret";
        let token = create_test_token("testuser", secret, -3600); // 1 hour ago

        let result = validate_token(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_secret() {
        let secret = "test-secret";
        let wrong_secret = "wrong-secret";
        let token = create_test_token("testuser", secret, 3600);

        let result = validate_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_malformed_token() {
        let secret = "test-secret";
        let token = "not.a.valid.jwt";

        let result = validate_token(token, secret);
        assert!(result.is_err());
    }
}
