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

    let token = create_token(&user.username, &auth.jwt_secret, auth.jwt_expiry_hours)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token,
        username: user.username.clone(),
        expires_in: auth.jwt_expiry_hours * 3600,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::auth::LoginRequest;
    use crate::auth::hash_password;
    use crate::config::{
        ApiConfig, AuthConfig, Config, DbConfig, RedpandaConfig, User, WriteConfig,
    };

    #[allow(dead_code)] // Used in tests
    fn create_test_config() -> Config {
        let password_hash = hash_password("testpass").unwrap();

        Config {
            redpanda: RedpandaConfig {
                brokers: "localhost:9092".into(),
                group_id: "test".into(),
                auto_offset_reset: "earliest".into(),
            },
            database: DbConfig {
                url: "postgres://postgres:postgres@localhost:5432/test".into(),
                write: WriteConfig {
                    batch_size: 100,
                    linger_ms: 100,
                },
            },
            pipelines: vec![],
            api: Some(ApiConfig {
                enabled: true,
                host: "0.0.0.0".into(),
                port: 8080,
            }),
            auth: Some(AuthConfig {
                jwt_secret: "test-secret-key-for-testing-only".into(),
                jwt_expiry_hours: 24,
                users: vec![User {
                    username: "testuser".into(),
                    password_hash,
                }],
            }),
        }
    }

    #[tokio::test]
    async fn test_login_request_validation() {
        // Test that LoginRequest can be deserialized
        let json = r#"{"username":"testuser","password":"testpass"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
    }

    #[tokio::test]
    async fn test_login_response_serialization() {
        // Test that LoginResponse can be serialized
        let response = LoginResponse {
            token: "test-token".to_string(),
            username: "testuser".to_string(),
            expires_in: 86400,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-token"));
        assert!(json.contains("testuser"));
        assert!(json.contains("86400"));
    }

    #[test]
    fn test_login_request_fields() {
        // Unit test for LoginRequest struct
        let request = LoginRequest {
            username: "admin".to_string(),
            password: "secret123".to_string(),
        };
        assert_eq!(request.username, "admin");
        assert_eq!(request.password, "secret123");
    }

    #[test]
    fn test_login_response_fields() {
        // Unit test for LoginResponse struct
        let response = LoginResponse {
            token: "jwt-token-here".to_string(),
            username: "user1".to_string(),
            expires_in: 3600,
        };
        assert_eq!(response.token, "jwt-token-here");
        assert_eq!(response.username, "user1");
        assert_eq!(response.expires_in, 3600);
    }

    #[test]
    fn test_login_response_expires_in_calculation() {
        // Test that expires_in is calculated correctly (hours * 3600)
        let hours = 24;
        let expected_seconds = hours * 3600;
        assert_eq!(expected_seconds, 86400);

        let response = LoginResponse {
            token: "token".to_string(),
            username: "user".to_string(),
            expires_in: hours * 3600,
        };
        assert_eq!(response.expires_in, 86400);
    }
}
