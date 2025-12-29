use crate::api::models::auth::{LoginRequest, LoginResponse};
use crate::auth::{jwt::create_token, password::verify_password};
use crate::config::Config;
use crate::db::DbPool;
use axum::{extract::State, http::StatusCode, response::Json};

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
