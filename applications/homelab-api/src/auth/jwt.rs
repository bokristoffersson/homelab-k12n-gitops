use alcoholic_jwt::{validate, Validation as JwksValidation, ValidationError, JWKS};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: String,           // username/subject
    pub exp: usize,            // expiration time (changed to usize for JWKS compatibility)
    pub iat: Option<usize>,    // issued at (optional)
    pub iss: Option<String>,   // issuer (for JWKS validation)
    pub email: Option<String>, // email (from Authentik)
}

// JWKS-based JWT Validator for RS256 tokens from Authentik
#[derive(Clone)]
#[allow(dead_code)]
pub struct JwtValidator {
    jwks: Arc<RwLock<JWKS>>,
    issuer: String,
}

impl JwtValidator {
    pub async fn new(jwks_url: &str, issuer: String) -> Result<Self, Box<dyn std::error::Error>> {
        let jwks = fetch_jwks(jwks_url).await?;
        Ok(Self {
            jwks: Arc::new(RwLock::new(jwks)),
            issuer,
        })
    }

    #[allow(dead_code)]
    pub async fn validate_token(&self, token: &str) -> Result<Claims, ValidationError> {
        let jwks = self.jwks.read().await;
        let validations = vec![
            JwksValidation::Issuer(self.issuer.clone()),
            JwksValidation::SubjectPresent,
        ];

        let kid = alcoholic_jwt::token_kid(token)
            .map_err(|_| ValidationError::InvalidSignature)?
            .ok_or(ValidationError::InvalidSignature)?;

        let jwk = jwks.find(&kid).ok_or(ValidationError::InvalidSignature)?;

        let valid_jwt = validate(token, jwk, validations)?;

        serde_json::from_value(valid_jwt.claims).map_err(|_| ValidationError::InvalidSignature)
    }
}

async fn fetch_jwks(url: &str) -> Result<JWKS, Box<dyn std::error::Error>> {
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
