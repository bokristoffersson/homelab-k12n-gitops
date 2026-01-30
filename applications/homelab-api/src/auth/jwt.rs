use alcoholic_jwt::{validate, Validation as JwksValidation, ValidationError, JWKS};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: String,           // username/subject
    pub exp: usize,            // expiration time
    pub iat: Option<usize>,    // issued at (optional)
    pub iss: Option<String>,   // issuer
    pub email: Option<String>, // email (from Authentik)
}

// Token introspection response from Authentik
#[derive(Debug, Clone, Deserialize)]
struct IntrospectionResponse {
    active: bool,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    exp: Option<usize>,
    #[serde(default)]
    iat: Option<usize>,
    #[serde(default)]
    iss: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
}

// Single issuer configuration
#[derive(Clone)]
struct IssuerEntry {
    name: String,
    issuer: String,
    jwks: Option<Arc<RwLock<JWKS>>>,
    introspection_url: Option<String>,
    // Client credentials for token introspection (RFC 7662 requires authentication)
    introspection_client_id: Option<String>,
    introspection_client_secret: Option<String>,
}

// Multi-issuer token validator for Authentik
// Supports both JWT tokens (via JWKS) and opaque tokens (via introspection)
#[derive(Clone)]
pub struct JwtValidator {
    issuers: Vec<IssuerEntry>,
    // Map from issuer URL to index for quick lookup
    issuer_index: HashMap<String, usize>,
    http_client: reqwest::Client,
}

impl JwtValidator {
    // Create a new multi-issuer validator
    pub async fn new_multi(
        configs: Vec<crate::config::IssuerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut issuers = Vec::new();
        let mut issuer_index = HashMap::new();
        let http_client = reqwest::Client::new();

        for (idx, config) in configs.into_iter().enumerate() {
            let jwks = if let Some(ref url) = config.jwks_url {
                match fetch_jwks(url).await {
                    Ok(jwks) => {
                        info!("Loaded JWKS for issuer '{}' from {}", config.name, url);
                        Some(Arc::new(RwLock::new(jwks)))
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load JWKS for issuer '{}': {} (will use introspection only)",
                            config.name, e
                        );
                        None
                    }
                }
            } else {
                None
            };

            // Validate introspection configuration: if URL is set, credentials are required
            if config.introspection_url.is_some() {
                if config.introspection_client_id.is_none()
                    || config.introspection_client_secret.is_none()
                {
                    return Err(format!(
                        "Issuer '{}': introspection_url requires introspection_client_id and introspection_client_secret",
                        config.name
                    ).into());
                }
                info!(
                    "Issuer '{}' configured for token introspection",
                    config.name
                );
            }

            issuers.push(IssuerEntry {
                name: config.name.clone(),
                issuer: config.issuer.clone(),
                jwks,
                introspection_url: config.introspection_url,
                introspection_client_id: config.introspection_client_id,
                introspection_client_secret: config.introspection_client_secret,
            });
            issuer_index.insert(config.issuer, idx);
        }

        Ok(Self {
            issuers,
            issuer_index,
            http_client,
        })
    }

    // Legacy constructor for single issuer (backwards compatibility)
    pub async fn new(
        jwks_url: &str,
        issuer: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let jwks = fetch_jwks(jwks_url).await?;
        let issuer_clone = issuer.clone();
        let mut issuer_index = HashMap::new();
        issuer_index.insert(issuer.clone(), 0);

        Ok(Self {
            issuers: vec![IssuerEntry {
                name: "default".to_string(),
                issuer: issuer_clone,
                jwks: Some(Arc::new(RwLock::new(jwks))),
                introspection_url: None,
                introspection_client_id: None,
                introspection_client_secret: None,
            }],
            issuer_index,
            http_client: reqwest::Client::new(),
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, ValidationError> {
        // Check if this looks like a JWT (has 3 parts separated by dots)
        let is_jwt = token.split('.').count() == 3;

        if is_jwt {
            // Try JWT validation first
            if let Ok(claims) = self.validate_jwt(token).await {
                return Ok(claims);
            }
        }

        // Try token introspection for opaque tokens or if JWT validation failed
        if let Ok(claims) = self.introspect_token(token).await {
            return Ok(claims);
        }

        warn!("Token validation failed with all methods");
        Err(ValidationError::InvalidSignature)
    }

    async fn validate_jwt(&self, token: &str) -> Result<Claims, ValidationError> {
        // First, try to extract the issuer from the token to find the right JWKS
        if let Some(iss) = extract_issuer_from_token(token) {
            if let Some(&idx) = self.issuer_index.get(&iss) {
                let entry = &self.issuers[idx];
                if entry.jwks.is_some() {
                    debug!("Validating JWT from issuer '{}' ({})", entry.name, iss);
                    return self.validate_jwt_with_issuer(token, entry).await;
                }
            } else {
                debug!("Unknown issuer in token: {}", iss);
            }
        }

        // If issuer extraction failed or issuer not found, try all issuers with JWKS
        debug!(
            "Trying all {} configured issuers for JWT validation",
            self.issuers.len()
        );
        for entry in &self.issuers {
            if entry.jwks.is_some() {
                match self.validate_jwt_with_issuer(token, entry).await {
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
        }

        Err(ValidationError::InvalidSignature)
    }

    async fn validate_jwt_with_issuer(
        &self,
        token: &str,
        entry: &IssuerEntry,
    ) -> Result<Claims, ValidationError> {
        let jwks = entry
            .jwks
            .as_ref()
            .ok_or(ValidationError::InvalidSignature)?;
        let jwks = jwks.read().await;
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

    async fn introspect_token(&self, token: &str) -> Result<Claims, ValidationError> {
        // Try introspection with all configured issuers that have introspection URLs
        for entry in &self.issuers {
            if let Some(ref url) = entry.introspection_url {
                debug!("Introspecting token with issuer '{}'", entry.name);
                match self.introspect_with_entry(token, url, entry).await {
                    Ok(claims) => {
                        debug!(
                            "Token introspection successful with issuer '{}'",
                            entry.name
                        );
                        return Ok(claims);
                    }
                    Err(e) => {
                        debug!(
                            "Token introspection failed with issuer '{}': {:?}",
                            entry.name, e
                        );
                    }
                }
            }
        }

        Err(ValidationError::InvalidSignature)
    }

    async fn introspect_with_entry(
        &self,
        token: &str,
        url: &str,
        entry: &IssuerEntry,
    ) -> Result<Claims, ValidationError> {
        // Build request with optional client credentials (RFC 7662 requires authentication)
        let mut request = self.http_client.post(url).form(&[("token", token)]);

        // Add Basic auth if client credentials are configured
        if let (Some(client_id), Some(client_secret)) = (
            &entry.introspection_client_id,
            &entry.introspection_client_secret,
        ) {
            request = request.basic_auth(client_id, Some(client_secret));
        } else {
            warn!(
                "No client credentials configured for introspection with issuer '{}'. \
                 RFC 7662 requires authentication - introspection may fail.",
                entry.name
            );
        }

        let response = request.send().await.map_err(|e| {
            warn!("Introspection request failed: {}", e);
            ValidationError::InvalidSignature
        })?;

        if !response.status().is_success() {
            debug!(
                "Introspection endpoint returned {} for issuer '{}'",
                response.status(),
                entry.name
            );
            return Err(ValidationError::InvalidSignature);
        }

        let introspection: IntrospectionResponse = response.json().await.map_err(|e| {
            warn!("Failed to parse introspection response: {}", e);
            ValidationError::InvalidSignature
        })?;

        if !introspection.active {
            debug!("Token is not active");
            return Err(ValidationError::InvalidSignature);
        }

        // Convert introspection response to Claims
        let sub = introspection
            .sub
            .or(introspection.username)
            .or(introspection.preferred_username)
            .ok_or_else(|| {
                warn!("Introspection response missing subject");
                ValidationError::InvalidSignature
            })?;

        Ok(Claims {
            sub,
            exp: introspection.exp.unwrap_or(0),
            iat: introspection.iat,
            iss: introspection.iss,
            email: introspection.email,
        })
    }

    #[allow(dead_code)]
    pub fn issuer_count(&self) -> usize {
        self.issuers.len()
    }
}

// Extract issuer from JWT without validating signature
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

async fn fetch_jwks(url: &str) -> Result<JWKS, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get(url).await?;
    let jwks: JWKS = res.json().await?;
    Ok(jwks)
}

// Legacy HS256 token creation (for local auth, if needed)
#[allow(dead_code)]
pub fn create_token(username: &str, secret: &str, expiry_hours: u64) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiry_hours as i64);

    let claims = Claims {
        sub: username.to_string(),
        exp: exp.timestamp() as usize,
        iat: Some(now.timestamp() as usize),
        iss: None,
        email: None,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| format!("Failed to create token: {}", e))
}

// Legacy HS256 token validation (for local auth, if needed)
#[allow(dead_code)]
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, String> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|e| format!("Invalid token: {}", e))?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_issuer_from_token() {
        // A sample JWT with iss claim (not a real token, just for testing structure)
        // Header: {"alg":"RS256","typ":"JWT"}
        // Payload: {"iss":"https://example.com/","sub":"user123","exp":9999999999}
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2V4YW1wbGUuY29tLyIsInN1YiI6InVzZXIxMjMiLCJleHAiOjk5OTk5OTk5OTl9.signature";

        let issuer = extract_issuer_from_token(token);
        assert_eq!(issuer, Some("https://example.com/".to_string()));
    }

    #[test]
    fn test_extract_issuer_invalid_token() {
        assert_eq!(extract_issuer_from_token("not-a-jwt"), None);
        assert_eq!(extract_issuer_from_token("only.two"), None);
    }

    #[test]
    fn test_is_jwt_detection() {
        // JWT has 3 parts separated by dots
        assert_eq!("a.b.c".split('.').count(), 3);
        // Opaque token doesn't
        assert_ne!("opaque-token-abc123".split('.').count(), 3);
    }
}
