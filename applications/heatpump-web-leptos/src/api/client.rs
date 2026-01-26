use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use wasm_bindgen::JsValue;

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
        let url = format!("{}{}", self.base_url, path);

        let response = Request::get(&url)
            .credentials(web_sys::RequestCredentials::Include) // Send cookies
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a PATCH request with a JSON body
    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);

        let response = Request::patch(&url)
            .credentials(web_sys::RequestCredentials::Include)
            .header("Content-Type", "application/json")
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
            // Redirect to login
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
    "https://heatpump.k12n.com".to_string()
}

/// Redirect to oauth2-proxy login
fn redirect_to_login() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(_location) = window.location().href() {
                let _ = window.location().set_href("/oauth2/sign_in");
            }
        }
    }
}
