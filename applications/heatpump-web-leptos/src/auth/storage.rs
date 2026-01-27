//! Token storage utilities using browser localStorage/sessionStorage

use gloo_storage::{LocalStorage, SessionStorage, Storage};
use serde::{Deserialize, Serialize};

const TOKEN_KEY: &str = "oauth_access_token";
const REFRESH_TOKEN_KEY: &str = "oauth_refresh_token";
const TOKEN_EXPIRY_KEY: &str = "oauth_token_expiry";
const USER_INFO_KEY: &str = "oauth_user_info";
const CODE_VERIFIER_KEY: &str = "oauth_code_verifier";
const STATE_KEY: &str = "oauth_state";

/// User information from OIDC userinfo endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub preferred_username: Option<String>,
}

/// Store access token in localStorage
pub fn store_access_token(token: &str) {
    let _ = LocalStorage::set(TOKEN_KEY, token);
}

/// Get access token from localStorage
pub fn get_access_token() -> Option<String> {
    LocalStorage::get(TOKEN_KEY).ok()
}

/// Store refresh token in localStorage
pub fn store_refresh_token(token: &str) {
    let _ = LocalStorage::set(REFRESH_TOKEN_KEY, token);
}

/// Get refresh token from localStorage
pub fn get_refresh_token() -> Option<String> {
    LocalStorage::get(REFRESH_TOKEN_KEY).ok()
}

/// Store token expiry timestamp
pub fn store_token_expiry(expiry_ms: u64) {
    let _ = LocalStorage::set(TOKEN_EXPIRY_KEY, expiry_ms.to_string());
}

/// Get token expiry timestamp
pub fn get_token_expiry() -> Option<u64> {
    LocalStorage::get::<String>(TOKEN_EXPIRY_KEY)
        .ok()
        .and_then(|s| s.parse().ok())
}

/// Store user info
pub fn store_user_info(user_info: &UserInfo) {
    let _ = LocalStorage::set(USER_INFO_KEY, user_info);
}

/// Get stored user info
pub fn get_user_info() -> Option<UserInfo> {
    LocalStorage::get(USER_INFO_KEY).ok()
}

/// Clear all auth tokens from storage
pub fn clear_tokens() {
    LocalStorage::delete(TOKEN_KEY);
    LocalStorage::delete(REFRESH_TOKEN_KEY);
    LocalStorage::delete(TOKEN_EXPIRY_KEY);
    LocalStorage::delete(USER_INFO_KEY);
}

/// Store PKCE code verifier in sessionStorage (temporary)
pub fn store_code_verifier(verifier: &str) {
    let _ = SessionStorage::set(CODE_VERIFIER_KEY, verifier);
}

/// Get and remove PKCE code verifier from sessionStorage
pub fn get_code_verifier() -> Option<String> {
    let verifier = SessionStorage::get(CODE_VERIFIER_KEY).ok();
    SessionStorage::delete(CODE_VERIFIER_KEY);
    verifier
}

/// Store OAuth state in sessionStorage (temporary)
pub fn store_oauth_state(state: &str) {
    let _ = SessionStorage::set(STATE_KEY, state);
}

/// Get and remove OAuth state from sessionStorage
pub fn get_oauth_state() -> Option<String> {
    let state = SessionStorage::get(STATE_KEY).ok();
    SessionStorage::delete(STATE_KEY);
    state
}

/// Check if user is authenticated (has valid non-expired token)
pub fn is_authenticated() -> bool {
    let token = get_access_token();
    let expiry = get_token_expiry();

    match (token, expiry) {
        (Some(_), Some(exp)) => {
            let now = js_sys::Date::now() as u64;
            // Consider expired if less than 5 minutes remaining
            let buffer_ms = 5 * 60 * 1000;
            now < exp.saturating_sub(buffer_ms)
        }
        _ => false,
    }
}

/// Check if token needs refresh (less than 10 minutes remaining)
pub fn needs_refresh() -> bool {
    if let Some(expiry) = get_token_expiry() {
        let now = js_sys::Date::now() as u64;
        let refresh_threshold = 10 * 60 * 1000;
        now > expiry.saturating_sub(refresh_threshold)
    } else {
        false
    }
}
