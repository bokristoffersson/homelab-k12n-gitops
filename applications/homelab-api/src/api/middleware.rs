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
// Fields are stored in request extensions for future use by handlers
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthenticatedUser {
    pub username: String,
    pub email: Option<String>,
    pub scopes: Vec<String>,
}

impl AuthenticatedUser {
    pub fn has_scope(&self, required: &str) -> bool {
        self.scopes.iter().any(|s| s == required)
    }
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
        // oauth2-proxy forwards the upstream access token; try to extract scopes from it
        // so downstream scope checks work even when the session-cookie flow is used.
        let scopes = request
            .headers()
            .get("x-auth-request-access-token")
            .and_then(|h| h.to_str().ok())
            .map(extract_scopes_from_jwt)
            .unwrap_or_default();
        request.extensions_mut().insert(AuthenticatedUser {
            username,
            email: oauth2_email,
            scopes,
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
        scopes: claims.scope,
    });

    Ok(next.run(request).await)
}

// Best-effort scope extraction from an already-validated upstream JWT.
// oauth2-proxy has already authenticated the session, so we trust the payload here
// for scope propagation only; signature validation remains Authentik's responsibility.
//
// Accepts three shapes, in priority order:
//   1. `scope`: space-separated string (RFC 8693 style)
//   2. `scope`: array of strings
//   3. Top-level boolean claims whose name contains `:` and value is `true`
//      (Authentik's scope-mapping `expression` emits scopes this way.)
fn extract_scopes_from_jwt(token: &str) -> Vec<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Vec::new();
    }
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let Ok(payload) = URL_SAFE_NO_PAD.decode(parts[1]) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&payload) else {
        return Vec::new();
    };
    match value.get("scope") {
        Some(serde_json::Value::String(s)) => {
            return s.split_whitespace().map(|p| p.to_string()).collect();
        }
        Some(serde_json::Value::Array(items)) => {
            return items
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect();
        }
        _ => {}
    }
    let serde_json::Value::Object(map) = value else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(k, v)| {
            if k.contains(':') && v.as_bool().unwrap_or(false) {
                Some(k.clone())
            } else {
                None
            }
        })
        .collect()
}

// Required scope, passed as middleware state so route builders can specify it per group.
#[derive(Clone)]
pub struct RequiredScope(pub &'static str);

// Scope-gating middleware. Runs after `require_jwt_auth`, so `AuthenticatedUser`
// is expected in request extensions. Missing scope returns 403; missing user returns 401.
pub async fn require_scope(
    State(RequiredScope(required)): State<RequiredScope>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_scope(required) {
        debug!(
            "scope check failed: user={} missing={}",
            user.username, required
        );
        return Err(StatusCode::FORBIDDEN);
    }

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
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    fn make_jwt_with_payload(payload: serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        format!("{header}.{body}.sig")
    }

    #[test]
    fn extract_scopes_reads_space_separated_string() {
        let token =
            make_jwt_with_payload(serde_json::json!({"scope": "read:energy read:heatpump"}));
        let scopes = extract_scopes_from_jwt(&token);
        assert!(scopes.contains(&"read:energy".to_string()));
        assert!(scopes.contains(&"read:heatpump".to_string()));
    }

    #[test]
    fn extract_scopes_reads_array() {
        let token =
            make_jwt_with_payload(serde_json::json!({"scope": ["read:energy", "read:heatpump"]}));
        let scopes = extract_scopes_from_jwt(&token);
        assert!(scopes.contains(&"read:energy".to_string()));
        assert!(scopes.contains(&"read:heatpump".to_string()));
    }

    #[test]
    fn extract_scopes_reads_top_level_boolean_claims() {
        // Shape Authentik actually emits: each scope is a top-level
        // boolean claim whose key contains ':'.
        let token = make_jwt_with_payload(serde_json::json!({
            "sub": "user-1",
            "read:energy": true,
            "read:heatpump": true,
            "email_verified": true,
            "write:plugs": true,
        }));
        let scopes = extract_scopes_from_jwt(&token);
        assert!(scopes.contains(&"read:energy".to_string()));
        assert!(scopes.contains(&"read:heatpump".to_string()));
        assert!(scopes.contains(&"write:plugs".to_string()));
        assert!(
            !scopes.contains(&"email_verified".to_string()),
            "non-scope boolean claim (no colon) must not be picked up"
        );
    }

    #[test]
    fn extract_scopes_returns_empty_on_invalid_token() {
        assert!(extract_scopes_from_jwt("not.a.jwt").is_empty());
        assert!(extract_scopes_from_jwt("onlyone").is_empty());
    }

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

    use axum::{routing::get, Router};
    use axum_test::TestServer;

    async fn seed_user(
        mut request: Request,
        next: Next,
        user: AuthenticatedUser,
    ) -> Result<Response, StatusCode> {
        request.extensions_mut().insert(user);
        Ok(next.run(request).await)
    }

    fn test_app(user: Option<AuthenticatedUser>, required: &'static str) -> Router {
        let mut router: Router = Router::new().route("/guarded", get(|| async { "ok" }));
        router = router.layer(axum::middleware::from_fn_with_state(
            RequiredScope(required),
            require_scope,
        ));
        if let Some(user) = user {
            router = router.layer(axum::middleware::from_fn(
                move |req: Request, next: Next| {
                    let user = user.clone();
                    async move { seed_user(req, next, user).await }
                },
            ));
        }
        router
    }

    #[tokio::test]
    async fn require_scope_returns_403_when_scope_missing() {
        let user = AuthenticatedUser {
            username: "alice".into(),
            email: None,
            scopes: vec!["read:heatpump".into()],
        };
        let server = TestServer::new(test_app(Some(user), "read:energy")).unwrap();
        let response = server.get("/guarded").await;
        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn require_scope_returns_200_when_scope_present() {
        let user = AuthenticatedUser {
            username: "alice".into(),
            email: None,
            scopes: vec!["read:energy".into(), "read:heatpump".into()],
        };
        let server = TestServer::new(test_app(Some(user), "read:energy")).unwrap();
        let response = server.get("/guarded").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn require_scope_returns_401_when_user_missing() {
        let server = TestServer::new(test_app(None, "read:energy")).unwrap();
        let response = server.get("/guarded").await;
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}
