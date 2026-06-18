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

// Middleware that authenticates native-app requests via a Bearer JWT validated
// against the issuer's JWKS. The Traefik route for this service only forwards
// requests carrying an `Authorization: Bearer ...` header, so there is no
// oauth2-proxy/ForwardAuth header path here (which would otherwise be a forgeable
// trust boundary for any in-cluster caller reaching the pod directly).
pub async fn require_jwt_auth(
    State((_, _, validator)): State<crate::auth::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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
