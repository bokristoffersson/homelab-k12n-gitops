use crate::auth;
use crate::ws::connection::handle_connection;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::Response,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::kafka::EnergyMessage;

#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub broadcast_tx: broadcast::Sender<EnergyMessage>,
    pub max_connections: usize,
}

impl AppState {
    pub fn new(
        jwt_secret: String,
        broadcast_tx: broadcast::Sender<EnergyMessage>,
        max_connections: usize,
    ) -> Self {
        Self {
            jwt_secret,
            broadcast_tx,
            max_connections,
        }
    }
}

/// Handle WebSocket upgrade request
/// Expects JWT token in Authorization header or X-Auth-Request-Access-Token header (from oauth2-proxy)
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    // Extract JWT token from headers
    // First try X-Auth-Request-Access-Token (from oauth2-proxy ForwardAuth)
    let token = if let Some(token_header) = headers.get("x-auth-request-access-token") {
        token_header.to_str().map_err(|_| {
            error!("Invalid X-Auth-Request-Access-Token header");
            StatusCode::UNAUTHORIZED
        })?
    } else if let Some(auth_header) = headers.get("authorization") {
        // Fall back to Authorization header
        let auth_str = auth_header.to_str().map_err(|_| {
            error!("Invalid Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

        // Extract token from "Bearer <token>" format
        auth_str
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                error!("Authorization header missing Bearer prefix");
                StatusCode::UNAUTHORIZED
            })?
    } else {
        error!("No authentication token found in headers");
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validate JWT token
    let claims = auth::validate_token(token, &state.jwt_secret).map_err(|e| {
        error!("JWT validation failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

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
