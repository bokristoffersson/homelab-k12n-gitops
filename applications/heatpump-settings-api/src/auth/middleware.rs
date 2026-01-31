use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

use crate::api::handlers::AppState;

/// Extract Bearer token from Authorization header
fn extract_bearer_token(auth_header: Option<&str>) -> Option<&str> {
    match auth_header {
        Some(header) if header.starts_with("Bearer ") => Some(&header[7..]),
        _ => None,
    }
}

pub async fn require_jwt_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let jwt_validator = match &state.jwt_validator {
        Some(validator) => validator,
        None => {
            debug!("JWT validation disabled, allowing request");
            return Ok(next.run(request).await);
        }
    };

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match extract_bearer_token(auth_header) {
        Some(token) => token,
        None => {
            warn!("Missing or invalid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    match jwt_validator.validate_token(token).await {
        Ok(claims) => {
            debug!("JWT validated for user: {}", claims.sub);
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("JWT validation failed: {:?}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token_valid() {
        let token = extract_bearer_token(Some("Bearer abc123xyz"));
        assert_eq!(token, Some("abc123xyz"));
    }

    #[test]
    fn test_extract_bearer_token_with_jwt() {
        let jwt = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyIn0.sig";
        let header = format!("Bearer {}", jwt);
        let token = extract_bearer_token(Some(&header));
        assert_eq!(token, Some(jwt));
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let token = extract_bearer_token(None);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let token = extract_bearer_token(Some("Basic dXNlcjpwYXNz"));
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_empty() {
        let token = extract_bearer_token(Some(""));
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_bearer_only() {
        // "Bearer " with nothing after
        let token = extract_bearer_token(Some("Bearer "));
        assert_eq!(token, Some(""));
    }

    #[test]
    fn test_extract_bearer_token_lowercase() {
        // "bearer" lowercase should not match
        let token = extract_bearer_token(Some("bearer abc123"));
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_no_space() {
        // "Bearerabc" without space should not match
        let token = extract_bearer_token(Some("Bearerabc123"));
        assert_eq!(token, None);
    }
}
