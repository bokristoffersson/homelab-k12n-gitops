//! API client with OAuth2 JWT authentication

use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use wasm_bindgen::JsValue;

use crate::auth::{get_access_token, is_authenticated, needs_refresh, OAuthService};

/// API error types
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Unauthorized - please log in")]
    Unauthorized,
}

impl From<gloo_net::Error> for ApiError {
    fn from(err: gloo_net::Error) -> Self {
        ApiError::Network(err.to_string())
    }
}

/// API client for making HTTP requests
///
/// Authentication uses OAuth2 Authorization Code Flow with PKCE.
/// JWT access tokens are stored in localStorage and included in
/// the Authorization header as Bearer tokens. Backend APIs validate
/// the JWT signature using Authentik's JWKS endpoint.
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    /// Create a new API client with the base URL from window.ENV
    pub fn new() -> Self {
        let base_url = get_api_url();
        Self { base_url }
    }

    /// Make a GET request and deserialize the response
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        // Check if token needs refresh before making request
        if needs_refresh() {
            let oauth = OAuthService::new();
            let _ = oauth.refresh_token().await;
        }

        let url = format!("{}{}", self.base_url, path);

        let mut request = Request::get(&url);

        // Add Authorization header if we have a token
        if let Some(token) = get_access_token() {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request.send().await?;

        self.handle_response(response).await
    }

    /// Make a PATCH request with a JSON body
    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        // Check if token needs refresh before making request
        if needs_refresh() {
            let oauth = OAuthService::new();
            let _ = oauth.refresh_token().await;
        }

        let url = format!("{}{}", self.base_url, path);

        let mut request = Request::patch(&url).header("Content-Type", "application/json");

        // Add Authorization header if we have a token
        if let Some(token) = get_access_token() {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .json(body)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle the HTTP response
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<T, ApiError> {
        let status = response.status();

        if status == 401 {
            // Token expired or invalid - trigger OAuth login
            redirect_to_login();
            return Err(ApiError::Unauthorized);
        }

        if !response.ok() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::Http { status, message });
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::Deserialization(e.to_string()))
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if user is authenticated
pub fn check_authenticated() -> bool {
    is_authenticated()
}

/// Get API URL from window.ENV or use default
fn get_api_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(env) = js_sys::Reflect::get(&window, &JsValue::from_str("ENV")) {
                if !env.is_undefined() {
                    if let Ok(api_url) = js_sys::Reflect::get(&env, &JsValue::from_str("API_URL")) {
                        if let Some(url) = api_url.as_string() {
                            return url;
                        }
                    }
                }
            }
        }
    }

    // Default fallback
    "https://heatpump-leptos.k12n.com".to_string()
}

/// Redirect to OAuth login flow
fn redirect_to_login() {
    #[cfg(target_arch = "wasm32")]
    {
        // Use spawn_local to call async login in sync context
        wasm_bindgen_futures::spawn_local(async {
            let oauth = OAuthService::new();
            oauth.login().await;
        });
    }
}
