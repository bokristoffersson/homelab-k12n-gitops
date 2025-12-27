use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
    Subscribe { streams: Vec<String> },
    Unsubscribe { streams: Vec<String> },
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerMessage {
    Data {
        stream: String,
        timestamp: String,
        data: Value,
    },
    Pong {
        timestamp: String,
    },
    Error {
        message: String,
        code: String,
    },
    Subscribed {
        streams: Vec<String>,
    },
    Unsubscribed {
        streams: Vec<String>,
    },
}

impl ServerMessage {
    pub fn data(stream: impl Into<String>, data: Value) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        ServerMessage::Data {
            stream: stream.into(),
            timestamp: now,
            data,
        }
    }

    pub fn pong() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        ServerMessage::Pong { timestamp: now }
    }

    pub fn error(message: impl Into<String>, code: impl Into<String>) -> Self {
        ServerMessage::Error {
            message: message.into(),
            code: code.into(),
        }
    }

    pub fn subscribed(streams: Vec<String>) -> Self {
        ServerMessage::Subscribed { streams }
    }

    pub fn unsubscribed(streams: Vec<String>) -> Self {
        ServerMessage::Unsubscribed { streams }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_subscribe_deserialization() {
        let json = r#"{"type": "subscribe", "streams": ["energy"]}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Subscribe { streams } => {
                assert_eq!(streams, vec!["energy"]);
            }
            _ => panic!("Expected Subscribe message"),
        }
    }

    #[test]
    fn test_client_message_ping_deserialization() {
        let json = r#"{"type": "ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        assert!(matches!(msg, ClientMessage::Ping));
    }

    #[test]
    fn test_server_message_data_serialization() {
        let data = serde_json::json!({"power": 1234});
        let msg = ServerMessage::data("energy", data);

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"data"#));
        assert!(json.contains(r#""stream":"energy"#));
        assert!(json.contains(r#""power":1234"#));
        assert!(json.contains(r#""timestamp""#));
    }

    #[test]
    fn test_server_message_pong_serialization() {
        let msg = ServerMessage::pong();

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"pong"#));
        assert!(json.contains(r#""timestamp""#));
    }

    #[test]
    fn test_server_message_error_serialization() {
        let msg = ServerMessage::error("Test error", "TEST_ERROR");

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error"#));
        assert!(json.contains(r#""message":"Test error"#));
        assert!(json.contains(r#""code":"TEST_ERROR"#));
    }
}
