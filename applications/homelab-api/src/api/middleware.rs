use crate::auth::jwt::validate_token;
use crate::config::Config;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

// User info extracted from authentication (either oauth2-proxy headers or JWT)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
    pub email: Option<String>,
}

// Middleware that validates Bearer tokens using the multi-issuer JwtValidator
// Supports both:
// 1. oauth2-proxy headers (X-Auth-Request-User) - for web apps with session cookies
// 2. Bearer tokens in Authorization header - for native apps
pub async fn require_jwt_auth(
    State((_, _, validator)): State<crate::auth::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // First, check for oauth2-proxy headers (web app session flow)
    let oauth2_user = request
        .headers()
        .get("x-auth-request-user")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let oauth2_email = request
        .headers()
        .get("x-auth-request-email")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let Some(username) = oauth2_user {
        debug!("Authenticated via oauth2-proxy: {}", username);
        request.extensions_mut().insert(AuthenticatedUser {
            username,
            email: oauth2_email,
        });
        return Ok(next.run(request).await);
    }

    // Second, try Bearer token authentication (native app flow)
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let token = match auth_header {
        Some(ref header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            debug!("No valid Authorization header found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Validate using JwtValidator (multi-issuer support)
    let validator = validator.as_ref().ok_or_else(|| {
        warn!("JWT validator not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let claims = validator.validate_token(token).await.map_err(|e| {
        debug!("JWT validation failed: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    debug!("Authenticated via JWT: sub={}", claims.sub);
    request.extensions_mut().insert(AuthenticatedUser {
        username: claims.sub,
        email: claims.email,
    });

    Ok(next.run(request).await)
}

// Legacy HS256 middleware (for backwards compatibility with local auth)
#[allow(dead_code)]
pub async fn require_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract config from extensions (set by router layer)
    let config = request
        .extensions()
        .get::<Config>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth = config
        .auth
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let secret = auth
        .jwt_secret
        .as_deref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    validate_token(token, secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::create_token;

    #[test]
    fn test_bearer_token_extraction() {
        // Test token extraction logic used in middleware
        let header1 = "Bearer token123";
        assert!(header1.starts_with("Bearer "));
        let token1 = &header1[7..];
        assert_eq!(token1, "token123");

        let header2 = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token2 = &header2[7..];
        assert_eq!(token2, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_bearer_token_extraction_edge_cases() {
        // Test edge cases for token extraction
        let header_without_bearer = "token123";
        assert!(!header_without_bearer.starts_with("Bearer "));

        let header_empty = "Bearer ";
        assert!(header_empty.starts_with("Bearer "));
        let token = &header_empty[7..];
        assert_eq!(token, "");

        let header_lowercase = "bearer token123";
        assert!(!header_lowercase.starts_with("Bearer "));
    }

    #[test]
    fn test_token_validation_logic() {
        // Test that token validation would work with correct secret
        let secret = "test-secret-key";
        let username = "testuser";
        let token = create_token(username, secret, 24).unwrap();

        // Token should be valid
        let validation_result = validate_token(&token, secret);
        assert!(validation_result.is_ok());
        assert_eq!(validation_result.unwrap().sub, username);

        // Token should be invalid with wrong secret
        let wrong_secret = "wrong-secret";
        let validation_result = validate_token(&token, wrong_secret);
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_middleware_error_codes() {
        // Test that we understand the error codes used
        assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), 401);
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 500);
    }
}
