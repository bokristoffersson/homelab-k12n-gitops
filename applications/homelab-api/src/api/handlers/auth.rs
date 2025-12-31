use crate::api::models::auth::{LoginRequest, LoginResponse, UserInfoResponse};
use crate::auth::{jwt::create_token, password::verify_password};
use crate::config::Config;
use crate::db::DbPool;
use axum::{extract::State, http::{HeaderMap, StatusCode}, response::Json};

pub async fn login(
    State((_pool, config)): State<(DbPool, Config)>,
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

/// Get user info from oauth2-proxy headers and return a JWT token
/// This endpoint is called by authenticated users (via oauth2-proxy) to get a JWT token for WebSocket auth
pub async fn user_info(
    State((_pool, config)): State<(DbPool, Config)>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>, StatusCode> {
    let auth = config
        .auth
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract username from X-Forwarded-User header (set by oauth2-proxy)
    let username = headers
        .get("x-forwarded-user")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract email from X-Forwarded-Email header (optional)
    let email = headers
        .get("x-forwarded-email")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let secret = auth
        .jwt_secret
        .as_deref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create JWT token for this user
    let token = create_token(username, secret, auth.jwt_expiry_hours)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UserInfoResponse {
        token,
        username: username.to_string(),
        email,
        expires_in: auth.jwt_expiry_hours * 3600,
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
