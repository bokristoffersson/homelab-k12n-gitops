//! OAuth2 Authorization Code Flow with PKCE implementation

use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen::JsValue;

use super::storage::{
    clear_tokens, get_code_verifier, get_oauth_state, get_refresh_token, store_access_token,
    store_code_verifier, store_oauth_state, store_refresh_token, store_token_expiry,
    store_user_info, UserInfo,
};

/// OAuth configuration from window.ENV
#[derive(Clone)]
pub struct OAuthConfig {
    pub authentik_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: String,
}

impl OAuthConfig {
    /// Load configuration from window.ENV
    pub fn from_env() -> Self {
        let (authentik_url, client_id, redirect_uri) = get_oauth_config_from_env();

        Self {
            authentik_url,
            client_id,
            redirect_uri,
            scopes: "openid profile email".to_string(),
        }
    }
}

/// Token response from Authentik
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: Option<String>,
    #[allow(dead_code)]
    scope: Option<String>,
}

/// OAuth service for authentication operations
pub struct OAuthService {
    config: OAuthConfig,
}

impl OAuthService {
    pub fn new() -> Self {
        Self {
            config: OAuthConfig::from_env(),
        }
    }

    /// Start OAuth2 authorization flow - redirects to Authentik
    pub async fn login(&self) {
        let (code_verifier, code_challenge) = generate_pkce().await;
        let state = generate_random_string(32);

        // Store PKCE verifier and state for callback validation
        store_code_verifier(&code_verifier);
        store_oauth_state(&state);

        // Build authorization URL
        let auth_url = format!(
            "{}/application/o/authorize/?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.config.authentik_url,
            js_sys::encode_uri_component(&self.config.client_id),
            js_sys::encode_uri_component(&self.config.redirect_uri),
            js_sys::encode_uri_component(&self.config.scopes),
            js_sys::encode_uri_component(&state),
            js_sys::encode_uri_component(&code_challenge),
        );

        // Redirect to Authentik
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(&auth_url);
        }
    }

    /// Handle OAuth2 callback - exchange code for tokens
    pub async fn handle_callback(&self, code: &str, state: &str) -> Result<UserInfo, String> {
        // Validate state to prevent CSRF
        let stored_state = get_oauth_state().ok_or("Missing stored state")?;
        if state != stored_state {
            return Err("Invalid state parameter - possible CSRF attack".to_string());
        }

        // Get PKCE code verifier
        let code_verifier = get_code_verifier().ok_or("Missing code verifier")?;

        // Exchange code for tokens
        let token_url = format!("{}/application/o/token/", self.config.authentik_url);

        let body = format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
            js_sys::encode_uri_component(code),
            js_sys::encode_uri_component(&self.config.redirect_uri),
            js_sys::encode_uri_component(&self.config.client_id),
            js_sys::encode_uri_component(&code_verifier),
        );

        let response = Request::post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            let error = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {}", error));
        }

        let token_data: TokenResponse = response.json().await.map_err(|e| e.to_string())?;

        // Store tokens
        store_access_token(&token_data.access_token);
        if let Some(ref refresh_token) = token_data.refresh_token {
            store_refresh_token(refresh_token);
        }

        // Calculate and store expiry time
        let now = js_sys::Date::now() as u64;
        let expiry = now + (token_data.expires_in * 1000);
        store_token_expiry(expiry);

        // Fetch user info
        let user_info = self.fetch_user_info(&token_data.access_token).await?;
        store_user_info(&user_info);

        Ok(user_info)
    }

    /// Fetch user info from Authentik userinfo endpoint
    async fn fetch_user_info(&self, access_token: &str) -> Result<UserInfo, String> {
        let userinfo_url = format!("{}/application/o/userinfo/", self.config.authentik_url);

        let response = Request::get(&userinfo_url)
            .header("Authorization", &format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err("Failed to fetch user info".to_string());
        }

        response.json().await.map_err(|e| e.to_string())
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self) -> Result<(), String> {
        let refresh_token = get_refresh_token().ok_or("No refresh token")?;

        let token_url = format!("{}/application/o/token/", self.config.authentik_url);

        let body = format!(
            "grant_type=refresh_token&refresh_token={}&client_id={}",
            js_sys::encode_uri_component(&refresh_token),
            js_sys::encode_uri_component(&self.config.client_id),
        );

        let response = Request::post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err("Token refresh failed".to_string());
        }

        let token_data: TokenResponse = response.json().await.map_err(|e| e.to_string())?;

        // Update stored tokens
        store_access_token(&token_data.access_token);
        if let Some(ref new_refresh_token) = token_data.refresh_token {
            store_refresh_token(new_refresh_token);
        }

        let now = js_sys::Date::now() as u64;
        let expiry = now + (token_data.expires_in * 1000);
        store_token_expiry(expiry);

        Ok(())
    }

    /// Logout - clear tokens and redirect to Authentik logout
    pub fn logout(&self) {
        clear_tokens();

        // Redirect to Authentik end-session endpoint
        let logout_url = format!(
            "{}/application/o/{}/end-session/?post_logout_redirect_uri={}",
            self.config.authentik_url,
            self.config.client_id,
            js_sys::encode_uri_component(&get_origin()),
        );

        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(&logout_url);
        }
    }
}

impl Default for OAuthService {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate PKCE code verifier and challenge
async fn generate_pkce() -> (String, String) {
    let code_verifier = generate_random_string(64);

    // Generate SHA-256 hash of verifier
    let encoder = web_sys::TextEncoder::new().unwrap();
    let data = encoder.encode_with_input(&code_verifier);

    // Convert to Uint8Array for SubtleCrypto API
    let data_array = js_sys::Uint8Array::from(data.as_slice());

    let crypto = web_sys::window().unwrap().crypto().unwrap();
    let subtle = crypto.subtle();

    let hash_promise = subtle
        .digest_with_str_and_buffer_source("SHA-256", &data_array)
        .unwrap();

    let hash = wasm_bindgen_futures::JsFuture::from(hash_promise)
        .await
        .unwrap();

    let hash_array = js_sys::Uint8Array::new(&hash);
    let hash_bytes: Vec<u8> = hash_array.to_vec();

    // Base64url encode the hash
    let code_challenge = base64url_encode(&hash_bytes);

    (code_verifier, code_challenge)
}

/// Generate cryptographically random string
fn generate_random_string(length: usize) -> String {
    let crypto = web_sys::window().unwrap().crypto().unwrap();
    let mut array = vec![0u8; length];
    crypto.get_random_values_with_u8_array(&mut array).unwrap();

    array
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
        .chars()
        .take(length)
        .collect()
}

/// Base64url encode bytes (RFC 4648)
fn base64url_encode(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Get OAuth config from window.ENV
fn get_oauth_config_from_env() -> (String, String, String) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(env) = js_sys::Reflect::get(&window, &JsValue::from_str("ENV")) {
                if !env.is_undefined() {
                    let authentik_url =
                        js_sys::Reflect::get(&env, &JsValue::from_str("AUTHENTIK_URL"))
                            .ok()
                            .and_then(|v| v.as_string())
                            .unwrap_or_else(|| "https://authentik.k12n.com".to_string());

                    let client_id =
                        js_sys::Reflect::get(&env, &JsValue::from_str("OAUTH_CLIENT_ID"))
                            .ok()
                            .and_then(|v| v.as_string())
                            .unwrap_or_else(|| "heatpump-web-leptos".to_string());

                    let redirect_uri =
                        js_sys::Reflect::get(&env, &JsValue::from_str("OAUTH_REDIRECT_URI"))
                            .ok()
                            .and_then(|v| v.as_string())
                            .unwrap_or_else(|| format!("{}/auth/callback", get_origin()));

                    return (authentik_url, client_id, redirect_uri);
                }
            }
        }
    }

    // Default fallback
    (
        "https://authentik.k12n.com".to_string(),
        "heatpump-web-leptos".to_string(),
        format!("{}/auth/callback", get_origin()),
    )
}

/// Get current origin (e.g., "https://heatpump-leptos.k12n.com")
fn get_origin() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "https://heatpump-leptos.k12n.com".to_string())
}
