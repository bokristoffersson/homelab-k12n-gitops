use crate::api::models::auth::{LoginRequest, LoginResponse, UserInfoResponse};
use crate::auth::{jwt::create_token, password::verify_password};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use tracing;

#[allow(dead_code)]
pub async fn login(
    State((_pool, config, _validator)): State<crate::auth::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let auth = config
        .auth
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = auth
        .users
        .iter()
        .find(|u| u.username == payload.username)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let secret = auth
        .jwt_secret
        .as_deref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = create_token(&user.username, secret, auth.jwt_expiry_hours)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token,
        username: user.username.clone(),
        expires_in: auth.jwt_expiry_hours * 3600,
    }))
}

/// Get user info and access token from oauth2-proxy headers
/// This endpoint returns the Authentik OIDC access token for WebSocket authentication
/// OAuth2-proxy sets X-Forwarded-Access-Token header with the original OIDC token
pub async fn user_info(
    State((_pool, _config, _validator)): State<crate::auth::AppState>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>, StatusCode> {
    // Extract username from X-Auth-Request-User header (set by oauth2-proxy)
    let username = headers
        .get("x-auth-request-user")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract email from X-Auth-Request-Email header (optional)
    let email = headers
        .get("x-auth-request-email")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Extract the Authentik OIDC access token from oauth2-proxy header
    let token = headers
        .get("x-auth-request-access-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            tracing::error!("X-Auth-Request-Access-Token header not found");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(UserInfoResponse {
        token: token.to_string(),
        username: username.to_string(),
        email,
        expires_in: 3600, // Authentik token expiry (typically 1 hour)
    }))
}

#[cfg(test)]
mod tests {
    use crate::api::models::auth::LoginRequest;

    #[tokio::test]
    async fn test_login_request_validation() {
        // Test that LoginRequest can be deserialized
        let json = r#"{"username":"testuser","password":"testpass"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
    }
}
