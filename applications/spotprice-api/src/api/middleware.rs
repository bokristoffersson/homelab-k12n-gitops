use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

// User info extracted from authentication (either oauth2-proxy headers or JWT).
// Stored in request extensions for handlers to read.
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

// Middleware that validates Bearer tokens using the multi-issuer JwtValidator.
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

    let validator = validator.as_ref().ok_or_else(|| {
        warn!("JWT validator not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let claims = validator.validate_token(token).await.map_err(|e| {
        debug!("JWT validation failed: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    debug!("Authenticated via JWT: sub={}", claims.sub);
    let scopes = claims.all_scopes();
    request.extensions_mut().insert(AuthenticatedUser {
        username: claims.sub,
        email: claims.email,
        scopes,
    });

    Ok(next.run(request).await)
}

// Best-effort scope extraction from an already-validated upstream JWT.
// Accepts these shapes, in priority order:
//   1. `scope` or `scp`: space-separated string (RFC 8693 style)
//   2. `scope` or `scp`: array of strings (Authelia JWT access tokens use `scp`, RFC 9068)
//   3. Top-level boolean claims whose name contains `:` and value is `true`
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
    for key in ["scope", "scp"] {
        match value.get(key) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use axum_test::TestServer;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    fn make_jwt_with_payload(payload: serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let body = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        format!("{header}.{body}.sig")
    }

    #[test]
    fn extract_scopes_reads_scp_array() {
        let token = make_jwt_with_payload(serde_json::json!({"scp": ["read:spotprice"]}));
        let scopes = extract_scopes_from_jwt(&token);
        assert!(scopes.contains(&"read:spotprice".to_string()));
    }

    #[test]
    fn extract_scopes_returns_empty_on_invalid_token() {
        assert!(extract_scopes_from_jwt("not.a.jwt").is_empty());
        assert!(extract_scopes_from_jwt("onlyone").is_empty());
    }

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
            scopes: vec!["read:energy".into()],
        };
        let server = TestServer::new(test_app(Some(user), "read:spotprice")).unwrap();
        let response = server.get("/guarded").await;
        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn require_scope_returns_200_when_scope_present() {
        let user = AuthenticatedUser {
            username: "alice".into(),
            email: None,
            scopes: vec!["read:spotprice".into()],
        };
        let server = TestServer::new(test_app(Some(user), "read:spotprice")).unwrap();
        let response = server.get("/guarded").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[tokio::test]
    async fn require_scope_returns_401_when_user_missing() {
        let server = TestServer::new(test_app(None, "read:spotprice")).unwrap();
        let response = server.get("/guarded").await;
        assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
    }
}
