// Integration tests for authentication module
// These tests verify the authentication flow end-to-end

use redpanda_sink::auth::{
    hash_password,
    jwt::{create_token, validate_token},
    verify_password,
};
use redpanda_sink::config::{
    ApiConfig, AuthConfig, Config, DbConfig, RedpandaConfig, User, WriteConfig,
};

#[test]
fn test_auth_flow_integration() {
    // Test the complete authentication flow:
    // 1. Hash a password
    // 2. Verify the password
    // 3. Create a JWT token
    // 4. Validate the JWT token

    let password = "test-password-123";
    let username = "testuser";
    let secret = "test-secret-key";
    let expiry_hours = 24;

    // Step 1: Hash password
    let password_hash = hash_password(password).unwrap();
    assert!(!password_hash.is_empty());

    // Step 2: Verify password
    assert!(verify_password(password, &password_hash).unwrap());
    assert!(!verify_password("wrong-password", &password_hash).unwrap());

    // Step 3: Create JWT token
    let token = create_token(username, secret, expiry_hours).unwrap();
    assert!(!token.is_empty());

    // Step 4: Validate JWT token
    let claims = validate_token(&token, secret).unwrap();
    assert_eq!(claims.sub, username);
}

#[test]
fn test_auth_config_integration() {
    // Test that auth config can be used with auth functions

    let password = "admin-password";
    let username = "admin";
    let password_hash = hash_password(password).unwrap();

    let auth_config = AuthConfig {
        jwt_secret: "my-secret-key".to_string(),
        jwt_expiry_hours: 12,
        users: vec![User {
            username: username.to_string(),
            password_hash: password_hash.clone(),
        }],
    };

    // Verify user password
    let user = auth_config.users.first().unwrap();
    assert!(verify_password(password, &user.password_hash).unwrap());

    // Create token using config values
    let token = create_token(
        &user.username,
        &auth_config.jwt_secret,
        auth_config.jwt_expiry_hours,
    )
    .unwrap();

    // Validate token using config secret
    let claims = validate_token(&token, &auth_config.jwt_secret).unwrap();
    assert_eq!(claims.sub, username);
}

#[test]
fn test_multiple_users_auth() {
    // Test authentication with multiple users

    let users_data = vec![
        ("user1", "password1"),
        ("user2", "password2"),
        ("admin", "admin123"),
    ];

    let mut users = Vec::new();
    for (username, password) in &users_data {
        let hash = hash_password(password).unwrap();
        users.push(User {
            username: username.to_string(),
            password_hash: hash,
        });
    }

    let auth_config = AuthConfig {
        jwt_secret: "shared-secret".to_string(),
        jwt_expiry_hours: 24,
        users,
    };

    // Test each user can authenticate
    for (username, password) in &users_data {
        let user = auth_config
            .users
            .iter()
            .find(|u| u.username == *username)
            .unwrap();

        // Verify password
        assert!(verify_password(password, &user.password_hash).unwrap());

        // Create and validate token
        let token = create_token(
            username,
            &auth_config.jwt_secret,
            auth_config.jwt_expiry_hours,
        )
        .unwrap();

        let claims = validate_token(&token, &auth_config.jwt_secret).unwrap();
        assert_eq!(claims.sub, *username);
    }
}

#[test]
fn test_token_expiry_integration() {
    // Test that tokens expire correctly

    let secret = "test-secret";
    let username = "testuser";

    // Create token with very short expiry (1 hour)
    let token_1h = create_token(username, secret, 1).unwrap();
    assert!(validate_token(&token_1h, secret).is_ok());

    // Create token with longer expiry (24 hours)
    let token_24h = create_token(username, secret, 24).unwrap();
    assert!(validate_token(&token_24h, secret).is_ok());

    // Both tokens should be valid immediately
    let claims_1h = validate_token(&token_1h, secret).unwrap();
    let claims_24h = validate_token(&token_24h, secret).unwrap();

    // 24h token should expire later than 1h token
    assert!(claims_24h.exp > claims_1h.exp);
}

#[test]
fn test_wrong_secret_rejection() {
    // Test that tokens created with one secret are rejected with another

    let username = "testuser";
    let secret1 = "secret1";
    let secret2 = "secret2";

    let token = create_token(username, secret1, 24).unwrap();

    // Should validate with correct secret
    assert!(validate_token(&token, secret1).is_ok());

    // Should reject with wrong secret
    assert!(validate_token(&token, secret2).is_err());
}

#[test]
fn test_password_hash_persistence() {
    // Test that password hashes can be stored and verified later
    // (simulating database storage)

    let password = "persistent-password";
    let hash = hash_password(password).unwrap();

    // Simulate storing hash (e.g., in database)
    let stored_hash = hash.clone();

    // Later, verify against stored hash
    assert!(verify_password(password, &stored_hash).unwrap());
    assert!(!verify_password("wrong", &stored_hash).unwrap());
}

#[test]
fn test_auth_error_handling() {
    // Test error handling in auth functions

    // Invalid hash should return error
    assert!(verify_password("password", "invalid-hash").is_err());
    assert!(verify_password("password", "").is_err());

    // Invalid token should return error
    assert!(validate_token("invalid.token.here", "secret").is_err());
    assert!(validate_token("", "secret").is_err());
    assert!(validate_token("not-even-a-token", "secret").is_err());
}

#[test]
fn test_auth_with_config_structure() {
    // Test creating a full Config with auth and using it

    let password = "config-test-password";
    let username = "configuser";
    let password_hash = hash_password(password).unwrap();

    let config = Config {
        redpanda: RedpandaConfig {
            brokers: "localhost:9092".to_string(),
            group_id: "test".to_string(),
            auto_offset_reset: "earliest".to_string(),
        },
        database: DbConfig {
            url: "postgres://localhost/test".to_string(),
            write: WriteConfig {
                batch_size: 100,
                linger_ms: 100,
            },
        },
        pipelines: vec![],
        api: Some(ApiConfig {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 8080,
        }),
        auth: Some(AuthConfig {
            jwt_secret: "config-secret".to_string(),
            jwt_expiry_hours: 48,
            users: vec![User {
                username: username.to_string(),
                password_hash,
            }],
        }),
    };

    // Verify we can use the auth config
    let auth = config.auth.as_ref().unwrap();
    let user = auth.users.first().unwrap();

    assert!(verify_password(password, &user.password_hash).unwrap());

    let token = create_token(&user.username, &auth.jwt_secret, auth.jwt_expiry_hours).unwrap();

    let claims = validate_token(&token, &auth.jwt_secret).unwrap();
    assert_eq!(claims.sub, username);
}
