use crate::auth::{self, jwt::JwtValidator};
use crate::ws::connection::handle_connection;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::kafka::EnergyMessage;

#[derive(Deserialize)]
pub struct WsParams {
    token: String,
}

/// Authentication method for JWT validation
#[derive(Clone)]
pub enum AuthMethod {
    /// JWKS-based RS256 validation (preferred for OIDC)
    Jwks(Arc<JwtValidator>),
    /// Legacy HS256 validation with shared secret
    Legacy(String),
}

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthMethod,
    pub broadcast_tx: broadcast::Sender<EnergyMessage>,
    pub max_connections: usize,
}

impl AppState {
    pub fn new(
        auth: AuthMethod,
        broadcast_tx: broadcast::Sender<EnergyMessage>,
        max_connections: usize,
    ) -> Self {
        Self {
            auth,
            broadcast_tx,
            max_connections,
        }
    }
}

/// Handle WebSocket upgrade request
/// Expects JWT token as query parameter: /ws/energy?token=xxx
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    // Validate JWT token using the configured authentication method
    let claims = match &state.auth {
        AuthMethod::Jwks(validator) => {
            // Use JWKS/RS256 validation for OIDC tokens from Authentik
            validator
                .validate_token(&params.token)
                .await
                .map_err(|e| {
                    error!("JWKS JWT validation failed: {:?}", e);
                    StatusCode::UNAUTHORIZED
                })?
        }
        AuthMethod::Legacy(secret) => {
            // Use legacy HS256 validation
            auth::validate_token(&params.token, secret).map_err(|e| {
                error!("Legacy JWT validation failed: {}", e);
                StatusCode::UNAUTHORIZED
            })?
        }
    };

    info!("WebSocket upgrade authorized for user: {}", claims.sub);

    // TODO: Check current connection count against max_connections
    // For now, we'll accept all connections

    let client_id = format!("{}_{}", claims.sub, uuid::Uuid::new_v4());
    let broadcast_rx = state.broadcast_tx.subscribe();

    // Upgrade to WebSocket
    Ok(ws.on_upgrade(move |socket: WebSocket| handle_connection(socket, broadcast_rx, client_id)))
}

/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let (tx, _rx) = broadcast::channel(100);
        let state = AppState::new("test-secret".to_string(), tx, 1000);

        assert_eq!(state.jwt_secret, "test-secret");
        assert_eq!(state.max_connections, 1000);
    }
}
