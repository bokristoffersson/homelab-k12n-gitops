use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

use crate::api::handlers::AppState;
use crate::auth::jwt::Claims;

/// Authentication context attached to the request after auth middleware succeeds.
///
/// Two variants reflect the two trusted paths into the service:
/// - `Proxy` = oauth2-proxy validated the session cookie upstream; we trust the proxy
///   and grant all scopes (no token is available to inspect).
/// - `Jwt` = a Bearer JWT was validated locally; scope checks consult the `Claims.scope`.
#[derive(Clone, Debug)]
pub enum AuthContext {
    /// Trusted proxy (oauth2-proxy) validated an upstream session. We grant all scopes
    /// under the existing network-perimeter trust model; the principal string is kept
    /// for downstream logging/audit.
    Proxy {
        #[allow(dead_code)]
        user: String,
    },
    Jwt(Claims),
}

impl AuthContext {
    pub fn has_scope(&self, required: &str) -> bool {
        match self {
            AuthContext::Proxy { .. } => true,
            AuthContext::Jwt(claims) => claims.has_scope(required),
        }
    }
}

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

    let user = headers
        .get("X-Auth-Request-User")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty())?;

    let email = headers
        .get("X-Auth-Request-Email")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty());

    match email {
        Some(email) => Some(format!("{} ({})", user, email)),
        None => Some(user.to_string()),
    }
}

pub async fn require_jwt_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(proxy_user) = is_authenticated_by_proxy(&request) {
        debug!("Request authenticated via oauth2-proxy for: {}", proxy_user);
        request
            .extensions_mut()
            .insert(AuthContext::Proxy { user: proxy_user });
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
            request.extensions_mut().insert(AuthContext::Jwt(claims));
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("JWT validation failed: {:?}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Middleware that enforces a single required scope.
///
/// Must be layered **after** `require_jwt_auth` so that an `AuthContext` is present in
/// the request extensions. Returns 403 when the authenticated principal lacks the scope.
pub async fn require_scope(
    required: &'static str,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let authorized = match request.extensions().get::<AuthContext>() {
        Some(auth) => auth.has_scope(required),
        None => {
            debug!(
                "No AuthContext found on request; scope '{}' check skipped",
                required
            );
            true
        }
    };

    if !authorized {
        warn!("Request missing required scope: {}", required);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request, middleware, routing::get, Router};
    use tower::ServiceExt;

    fn make_request_with_headers(headers: Vec<(&str, &str)>) -> Request<Body> {
        let mut builder = Request::builder().uri("/test").method("GET");
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn jwt_claims(scope: Option<&str>) -> Claims {
        Claims {
            sub: "user".into(),
            exp: 0,
            iat: None,
            iss: None,
            email: None,
            scope: scope.map(str::to_owned),
        }
    }

    async fn inject_context(
        ctx: Option<AuthContext>,
    ) -> impl Fn(
        Request<Body>,
        Next,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>,
    > + Clone {
        move |mut req: Request<Body>, next: Next| {
            let ctx = ctx.clone();
            Box::pin(async move {
                if let Some(ctx) = ctx {
                    req.extensions_mut().insert(ctx);
                }
                Ok(next.run(req).await)
            })
        }
    }

    async fn run_with_scope(
        ctx: Option<AuthContext>,
        scope: &'static str,
    ) -> axum::http::StatusCode {
        let injector = inject_context(ctx).await;
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                require_scope(scope, req, next)
            }))
            .layer(middleware::from_fn(injector));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        app.oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn require_scope_allows_matching_jwt_scope() {
        let ctx = AuthContext::Jwt(jwt_claims(Some("read:plugs write:plugs")));
        let status = run_with_scope(Some(ctx), "read:plugs").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn require_scope_rejects_missing_jwt_scope() {
        let ctx = AuthContext::Jwt(jwt_claims(Some("read:heatpump")));
        let status = run_with_scope(Some(ctx), "write:plugs").await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn require_scope_rejects_empty_scope_claim() {
        let ctx = AuthContext::Jwt(jwt_claims(None));
        let status = run_with_scope(Some(ctx), "read:plugs").await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn require_scope_allows_proxy_auth() {
        let ctx = AuthContext::Proxy {
            user: "testuser".into(),
        };
        let status = run_with_scope(Some(ctx), "write:plugs").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn require_scope_allows_when_no_auth_context() {
        // Simulates JWT validator disabled: no AuthContext present, scope check skipped.
        let status = run_with_scope(None, "read:plugs").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn require_scope_returns_forbidden_body() {
        let ctx = AuthContext::Jwt(jwt_claims(Some("read:heatpump")));
        let injector = inject_context(Some(ctx)).await;
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(middleware::from_fn(|req, next| {
                require_scope("write:plugs", req, next)
            }))
            .layer(middleware::from_fn(injector));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        assert!(bytes.is_empty());
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
        let token = extract_bearer_token(Some("Bearer "));
        assert_eq!(token, Some(""));
    }

    #[test]
    fn test_extract_bearer_token_lowercase() {
        let token = extract_bearer_token(Some("bearer abc123"));
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_no_space() {
        let token = extract_bearer_token(Some("Bearerabc123"));
        assert_eq!(token, None);
    }
}
