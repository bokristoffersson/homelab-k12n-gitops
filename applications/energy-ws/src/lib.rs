pub mod auth;
pub mod config;
pub mod error;
pub mod kafka;
pub mod ws;

// Re-export commonly used items
pub use config::Config;
pub use error::{AppError, Result};
pub use kafka::EnergyMessage;
pub use ws::{AppState, ClientMessage, ServerMessage};
