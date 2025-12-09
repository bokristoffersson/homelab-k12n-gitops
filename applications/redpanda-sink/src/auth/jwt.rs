use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // username
    pub exp: i64,     // expiration time
    pub iat: i64,     // issued at
}

pub fn create_token(username: &str, secret: &str, expiry_hours: u64) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiry_hours as i64);
    
    let claims = Claims {
        sub: username.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| format!("Failed to create token: {}", e))
}

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
            exp: past_exp.timestamp(),
            iat: (now - Duration::hours(2)).timestamp(),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        ).unwrap();
        
        // Validation should fail due to expiration
        assert!(validate_token(&token, secret).is_err());
    }
}


