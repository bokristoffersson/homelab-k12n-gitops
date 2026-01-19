use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub token: String,
    pub username: String,
    pub email: Option<String>,
    pub expires_in: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_creation() {
        let request = LoginRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        assert_eq!(request.username, "testuser");
        assert_eq!(request.password, "testpass");
    }

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username":"admin","password":"secret123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.username, "admin");
        assert_eq!(request.password, "secret123");
    }

    #[test]
    fn test_login_request_round_trip() {
        // Test that we can deserialize what we expect to receive
        let json = r#"{"username":"user1","password":"pass1"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "user1");
        assert_eq!(request.password, "pass1");
    }

    #[test]
    fn test_login_response_creation() {
        let response = LoginResponse {
            token: "jwt-token-here".to_string(),
            username: "testuser".to_string(),
            expires_in: 86400,
        };

        assert_eq!(response.token, "jwt-token-here");
        assert_eq!(response.username, "testuser");
        assert_eq!(response.expires_in, 86400);
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
            username: "admin".to_string(),
            expires_in: 3600,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(json.contains("admin"));
        assert!(json.contains("3600"));
    }

    #[test]
    fn test_login_response_expires_in_calculation() {
        // Test that expires_in matches expected format (seconds)
        let hours = 24;
        let seconds = hours * 3600;

        let response = LoginResponse {
            token: "token".to_string(),
            username: "user".to_string(),
            expires_in: seconds,
        };

        assert_eq!(response.expires_in, 86400);
    }

    #[test]
    fn test_login_request_empty_fields() {
        let request = LoginRequest {
            username: "".to_string(),
            password: "".to_string(),
        };

        // Should still be valid struct
        assert_eq!(request.username, "");
        assert_eq!(request.password, "");
    }

    #[test]
    fn test_login_request_special_characters() {
        let request = LoginRequest {
            username: "user@example.com".to_string(),
            password: "p@ssw0rd!123".to_string(),
        };

        assert_eq!(request.username, "user@example.com");
        assert_eq!(request.password, "p@ssw0rd!123");
    }
}
