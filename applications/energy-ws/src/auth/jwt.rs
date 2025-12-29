use crate::error::{AppError, Result};
use alcoholic_jwt::{validate, Validation as JwksValidation, ValidationError, JWKS};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject (username)
    pub exp: usize,            // Expiration time
    pub iat: Option<usize>,    // Issued at (optional)
    pub iss: Option<String>,   // Issuer (for JWKS validation)
    pub email: Option<String>, // Email (from Authentik)
}

// JWKS-based JWT Validator for RS256 tokens from Authentik
#[derive(Clone)]
pub struct JwtValidator {
    jwks: Arc<RwLock<JWKS>>,
    issuer: String,
}

impl JwtValidator {
    pub async fn new(
        jwks_url: &str,
        issuer: String,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let jwks = fetch_jwks(jwks_url).await?;
        Ok(Self {
            jwks: Arc::new(RwLock::new(jwks)),
            issuer,
        })
    }

    pub async fn validate_token(
        &self,
        token: &str,
    ) -> std::result::Result<Claims, ValidationError> {
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

async fn fetch_jwks(url: &str) -> std::result::Result<JWKS, Box<dyn std::error::Error>> {
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
