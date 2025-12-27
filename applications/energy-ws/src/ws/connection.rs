use crate::kafka::EnergyMessage;
use crate::ws::protocol::{ClientMessage, ServerMessage};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::collections::HashSet;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Handle a WebSocket connection
pub async fn handle_connection(
    socket: WebSocket,
    broadcast_rx: broadcast::Receiver<EnergyMessage>,
    client_id: String,
) {
    info!("WebSocket client connected: {}", client_id);

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Track which streams the client is subscribed to
    let mut subscribed_streams: HashSet<String> = HashSet::new();

    // Clone receiver for the broadcast task
    let mut rx = broadcast_rx.resubscribe();

    // Clone client_id for tasks
    let send_client_id = client_id.clone();
    let recv_client_id = client_id.clone();

    // Spawn task to receive broadcasts from Kafka and send to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(energy_msg) = rx.recv().await {
            // Create server message
            let server_msg = ServerMessage::data("energy", energy_msg);

            // Serialize to JSON
            let json = match serde_json::to_string(&server_msg) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            // Send to WebSocket client
            if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                error!("Failed to send message to WebSocket: {}", e);
                break;
            }
        }
    });

    // Handle incoming messages from WebSocket client
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg_result) = ws_receiver.next().await {
            let msg = match msg_result {
                Ok(m) => m,
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            };

            match msg {
                Message::Text(text) => {
                    // Parse client message
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => match client_msg {
                            ClientMessage::Subscribe { streams } => {
                                info!("Client {} subscribing to: {:?}", recv_client_id, streams);
                                for stream in &streams {
                                    subscribed_streams.insert(stream.clone());
                                }
                                // Send confirmation (implementation would send this back)
                                debug!("Subscribed to streams: {:?}", subscribed_streams);
                            }
                            ClientMessage::Unsubscribe { streams } => {
                                info!("Client {} unsubscribing from: {:?}", recv_client_id, streams);
                                for stream in &streams {
                                    subscribed_streams.remove(stream);
                                }
                                debug!("Remaining subscriptions: {:?}", subscribed_streams);
                            }
                            ClientMessage::Ping => {
                                debug!("Received ping from client {}", recv_client_id);
                                // Send pong response (implementation would send this back)
                            }
                        },
                        Err(e) => {
                            warn!("Failed to parse client message: {}", e);
                        }
                    }
                }
                Message::Close(_) => {
                    info!("Client {} closed connection", recv_client_id);
                    break;
                }
                Message::Ping(_) | Message::Pong(_) => {
                    // Axum handles ping/pong automatically
                }
                Message::Binary(_) => {
                    warn!("Received unexpected binary message from client {}", recv_client_id);
                }
            }
        }

        subscribed_streams
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            info!("Send task completed for client {}", send_client_id);
            recv_task.abort();
        }
        subscriptions = &mut recv_task => {
            info!("Receive task completed for client {}", client_id);
            send_task.abort();
            if let Ok(subs) = subscriptions {
                debug!("Client {} was subscribed to: {:?}", client_id, subs);
            }
        }
    }

    info!("WebSocket client disconnected: {}", client_id);
}
