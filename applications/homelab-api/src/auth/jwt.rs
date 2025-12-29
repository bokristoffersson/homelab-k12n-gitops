use alcoholic_jwt::{validate, Validation as JwksValidation, ValidationError, JWKS};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // username/subject
    pub exp: usize,            // expiration time (changed to usize for JWKS compatibility)
    pub iat: Option<usize>,    // issued at (optional)
    pub iss: Option<String>,   // issuer (for JWKS validation)
    pub email: Option<String>, // email (from Authentik)
}

// JWKS-based JWT Validator for RS256 tokens from Authentik
#[derive(Clone)]
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
    fn test_create_and_validate_token() {
        let secret = "test-secret-key";
        let username = "testuser";
        let expiry_hours = 24;

        let token = create_token(username, secret, expiry_hours).unwrap();
        assert!(!token.is_empty());

        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.sub, username);
    }

    #[test]
    fn test_validate_token_wrong_secret() {
        let secret = "test-secret-key";
        let wrong_secret = "wrong-secret";
        let username = "testuser";

        let token = create_token(username, secret, 24).unwrap();
        assert!(validate_token(&token, wrong_secret).is_err());
    }

    #[test]
    fn test_validate_token_expired() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let secret = "test-secret-key";
        let username = "testuser";

        // Create a token with an expiration time in the past
        let now = Utc::now();
        let past_exp = now - Duration::hours(1);

        let claims = Claims {
            sub: username.to_string(),
            exp: past_exp.timestamp() as usize,
            iat: Some((now - Duration::hours(2)).timestamp() as usize),
            iss: None,
            email: None,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        // Validation should fail due to expiration
        assert!(validate_token(&token, secret).is_err());
    }

    #[test]
    fn test_create_token_with_empty_secret() {
        let secret = "";
        let username = "testuser";

        // Empty secret should still work (though not recommended)
        let result = create_token(username, secret, 24);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_token_with_empty_username() {
        let secret = "test-secret-key";
        let username = "";

        // Empty username should work
        let token = create_token(username, secret, 24).unwrap();
        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.sub, "");
    }

    #[test]
    fn test_create_token_with_zero_expiry() {
        let secret = "test-secret-key";
        let username = "testuser";

        // Token with 0 hours expiry should still be created
        let token = create_token(username, secret, 0).unwrap();
        // It might be immediately expired, but creation should succeed
        assert!(!token.is_empty());
    }

    #[test]
    fn test_validate_token_malformed() {
        let secret = "test-secret-key";

        // Test with completely invalid token string
        assert!(validate_token("not.a.valid.token", secret).is_err());

        // Test with empty string
        assert!(validate_token("", secret).is_err());

        // Test with only two parts (missing signature)
        assert!(validate_token(
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0In0",
            secret
        )
        .is_err());
    }

    #[test]
    fn test_token_claims_include_timestamps() {
        let secret = "test-secret-key";
        let username = "testuser";
        let expiry_hours = 24;

        let token = create_token(username, secret, expiry_hours).unwrap();
        let claims = validate_token(&token, secret).unwrap();

        // Verify timestamps are set
        assert!(!claims.sub.is_empty());
        assert!(claims.exp > 0);
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_token_different_expiry_hours() {
        let secret = "test-secret-key";
        let username = "testuser";

        // Create tokens with different expiry times
        let token_1h = create_token(username, secret, 1).unwrap();
        let token_24h = create_token(username, secret, 24).unwrap();
        let token_168h = create_token(username, secret, 168).unwrap();

        // All should be valid
        assert!(validate_token(&token_1h, secret).is_ok());
        assert!(validate_token(&token_24h, secret).is_ok());
        assert!(validate_token(&token_168h, secret).is_ok());

        // Tokens should be different
        assert_ne!(token_1h, token_24h);
        assert_ne!(token_24h, token_168h);
    }

    #[test]
    fn test_token_username_preserved() {
        let secret = "test-secret-key";
        let usernames = vec!["user1", "admin", "test-user-123", "user@example.com"];

        for username in usernames {
            let token = create_token(username, secret, 24).unwrap();
            let claims = validate_token(&token, secret).unwrap();
            assert_eq!(claims.sub, username);
        }
    }
}
