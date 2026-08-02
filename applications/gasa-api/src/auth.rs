use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

use crate::handlers::AppState;

/// Authenticated user attached to the request after the auth middleware succeeds.
///
/// # Security Model
/// Every external request to /api/* passes through the oauth2-proxy ForwardAuth
/// middleware in Traefik before reaching this service. oauth2-proxy validates the
/// user's session cookie against Authelia and injects X-Auth-Request-User /
/// X-Auth-Request-Email. This service is only reachable inside the cluster, so
/// those headers are trusted. Requests without them are rejected with 401.
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

/// Raw identity from the oauth2-proxy headers. The header user is Authelia's
/// opaque sub UUID; Config::username_for turns it into a display username.
fn user_from_headers(request: &Request<Body>) -> Option<(String, Option<String>)> {
    let headers = request.headers();

    let user = headers
        .get("X-Auth-Request-User")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty())?;

    let email = headers
        .get("X-Auth-Request-Email")
        .and_then(|h| h.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_owned);

    Some((user.to_string(), email))
}

pub async fn require_proxy_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let identity = user_from_headers(&request).or_else(|| {
        state
            .config
            .dev_user
            .as_ref()
            .map(|dev| (dev.username.clone(), Some(dev.email.clone())))
    });

    match identity {
        Some((header_user, email)) => {
            let username = state.config.username_for(&header_user, email.as_deref());
            let is_admin = state.config.is_admin(email.as_deref());
            request.extensions_mut().insert(CurrentUser {
                username,
                email,
                is_admin,
            });
            Ok(next.run(request).await)
        }
        None => {
            warn!("Request missing oauth2-proxy identity headers");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(headers: Vec<(&str, &str)>) -> Request<Body> {
        let mut builder = Request::builder().uri("/test").method("GET");
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn test_user_and_email_extracted() {
        let request = make_request(vec![
            ("X-Auth-Request-User", "erik"),
            ("X-Auth-Request-Email", "erik@example.com"),
        ]);
        assert_eq!(
            user_from_headers(&request),
            Some(("erik".to_string(), Some("erik@example.com".to_string())))
        );
    }

    #[test]
    fn test_user_without_email() {
        let request = make_request(vec![("X-Auth-Request-User", "erik")]);
        assert_eq!(
            user_from_headers(&request),
            Some(("erik".to_string(), None))
        );
    }

    #[test]
    fn test_missing_user_header_rejected() {
        let request = make_request(vec![("X-Auth-Request-Email", "erik@example.com")]);
        assert_eq!(user_from_headers(&request), None);
    }

    #[test]
    fn test_empty_user_header_rejected() {
        let request = make_request(vec![
            ("X-Auth-Request-User", ""),
            ("X-Auth-Request-Email", "erik@example.com"),
        ]);
        assert_eq!(user_from_headers(&request), None);
    }

    #[test]
    fn test_empty_email_treated_as_absent() {
        let request = make_request(vec![
            ("X-Auth-Request-User", "erik"),
            ("X-Auth-Request-Email", ""),
        ]);
        assert_eq!(
            user_from_headers(&request),
            Some(("erik".to_string(), None))
        );
    }
}
