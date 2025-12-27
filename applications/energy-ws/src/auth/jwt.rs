use crate::error::{AppError, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (username)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
}

/// Validate a JWT token and return the claims if valid
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
            iat: now,
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
