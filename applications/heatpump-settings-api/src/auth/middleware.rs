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

/// Check if request was authenticated by oauth2-proxy.
///
/// # Security Model
/// This function trusts requests that have been validated by oauth2-proxy, which runs as
/// a Traefik ForwardAuth middleware. The security relies on:
/// 1. Traefik middleware enforcement - all external requests to protected routes must pass
///    through oauth2-proxy-auth middleware before reaching this service
/// 2. Network isolation - this service is only accessible within the Kubernetes cluster
/// 3. Header validation - we require X-Auth-Request-User header to be present
///
/// oauth2-proxy sets these headers after validating the user's session cookie against Authentik.
fn is_authenticated_by_proxy(request: &Request<Body>) -> Option<String> {
    let headers = request.headers();

    // Require X-Auth-Request-User header (email is optional)
    // This matches homelab-api behavior for consistency
    let user = headers
        .get("X-Auth-Request-User")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty())?;

    let email = headers
        .get("X-Auth-Request-Email")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty());

    // User header present and non-empty - this request came through oauth2-proxy
    match email {
        Some(email) => Some(format!("{} ({})", user, email)),
        None => Some(user.to_string()),
    }
}

pub async fn require_jwt_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for oauth2-proxy authentication headers
    // Requires BOTH X-Auth-Request-User AND X-Auth-Request-Email to be present
    if let Some(proxy_user) = is_authenticated_by_proxy(&request) {
        debug!("Request authenticated via oauth2-proxy for: {}", proxy_user);
        return Ok(next.run(request).await);
    }

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
    use axum::http::Request;

    fn make_request_with_headers(headers: Vec<(&str, &str)>) -> Request<Body> {
        let mut builder = Request::builder().uri("/test").method("GET");
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn test_proxy_auth_with_both_headers() {
        let request = make_request_with_headers(vec![
            ("X-Auth-Request-User", "testuser"),
            ("X-Auth-Request-Email", "test@example.com"),
        ]);
        let result = is_authenticated_by_proxy(&request);
        assert_eq!(result, Some("testuser (test@example.com)".to_string()));
    }

    #[test]
    fn test_proxy_auth_user_only() {
        // Email is optional - user header alone should work
        let request = make_request_with_headers(vec![("X-Auth-Request-User", "testuser")]);
        let result = is_authenticated_by_proxy(&request);
        assert_eq!(result, Some("testuser".to_string()));
    }

    #[test]
    fn test_proxy_auth_missing_user() {
        let request = make_request_with_headers(vec![("X-Auth-Request-Email", "test@example.com")]);
        let result = is_authenticated_by_proxy(&request);
        assert_eq!(result, None);
    }

    #[test]
    fn test_proxy_auth_empty_user() {
        let request = make_request_with_headers(vec![
            ("X-Auth-Request-User", ""),
            ("X-Auth-Request-Email", "test@example.com"),
        ]);
        let result = is_authenticated_by_proxy(&request);
        assert_eq!(result, None);
    }

    #[test]
    fn test_proxy_auth_empty_email() {
        // Empty email should be treated as missing - user alone works
        let request = make_request_with_headers(vec![
            ("X-Auth-Request-User", "testuser"),
            ("X-Auth-Request-Email", ""),
        ]);
        let result = is_authenticated_by_proxy(&request);
        assert_eq!(result, Some("testuser".to_string()));
    }

    #[test]
    fn test_proxy_auth_no_headers() {
        let request = make_request_with_headers(vec![]);
        let result = is_authenticated_by_proxy(&request);
        assert_eq!(result, None);
    }

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
