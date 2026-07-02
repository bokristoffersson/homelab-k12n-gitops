//! Token storage utilities using browser localStorage/sessionStorage

use gloo_storage::{LocalStorage, SessionStorage, Storage};

const TOKEN_KEY: &str = "oauth_access_token";
const REFRESH_TOKEN_KEY: &str = "oauth_refresh_token";
const TOKEN_EXPIRY_KEY: &str = "oauth_token_expiry";
const CODE_VERIFIER_KEY: &str = "oauth_code_verifier";
const STATE_KEY: &str = "oauth_state";

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

/// Store PKCE code verifier in sessionStorage (temporary)
pub fn store_code_verifier(verifier: &str) {
    let _ = SessionStorage::set(CODE_VERIFIER_KEY, verifier);
}

/// Store OAuth state in sessionStorage (temporary)
pub fn store_oauth_state(state: &str) {
    let _ = SessionStorage::set(STATE_KEY, state);
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
