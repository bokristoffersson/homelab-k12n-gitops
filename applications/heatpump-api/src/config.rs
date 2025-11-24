use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Build DATABASE_URL from components if not set directly
        let database_url = if let Ok(url) = env::var("DATABASE_URL") {
            url
        } else {
            // Build from components
            let host = env::var("DATABASE_HOST")
                .unwrap_or_else(|_| "localhost".to_string());
            let port = env::var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string());
            let name = env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "postgres".to_string());
            let user = env::var("DATABASE_USER")
                .expect("DATABASE_USER must be set if DATABASE_URL is not set");
            let password = env::var("DATABASE_PASSWORD")
                .expect("DATABASE_PASSWORD must be set if DATABASE_URL is not set");
            
            format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, name)
        };
        
        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        let port = env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);

        Ok(Config {
            database: DatabaseConfig {
                url: database_url,
                max_connections: Some(max_connections),
            },
            server: ServerConfig { host, port },
        })
    }
}

