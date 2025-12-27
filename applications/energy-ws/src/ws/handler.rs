use crate::auth;
use crate::ws::connection::handle_connection;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::Response,
};
use std::collections::HashMap;
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
/// Expects JWT token in query parameter: /ws/energy?token=<JWT_TOKEN>
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    // Extract JWT token from query parameter
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;

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
