use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub kafka: KafkaConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    pub consumer_group: String,
    pub auto_offset_reset: String,
    pub session_timeout_ms: u32,
    pub enable_auto_commit: bool,
    pub auto_commit_interval_ms: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    /// Load configuration from file with environment variable substitution
    pub fn load() -> Result<Self> {
        let config_path =
            env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.yaml".to_string());

        tracing::info!("Loading configuration from: {}", config_path);

        // Read config file
        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path))?;

        // Substitute environment variables
        let config_content = substitute_env_vars(&config_content)?;

        // Parse YAML
        let config: Config =
            serde_yaml::from_str(&config_content).context("Failed to parse config YAML")?;

        tracing::info!("Configuration loaded successfully");
        Ok(config)
    }

    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn api_bind_address(&self) -> String {
        format!("{}:{}", self.api.host, self.api.port)
    }

    pub fn kafka_brokers(&self) -> String {
        self.kafka.brokers.join(",")
    }
}

/// Substitute environment variables in format $(VAR_NAME)
fn substitute_env_vars(content: &str) -> Result<String> {
    let mut result = content.to_string();
    let re = regex::Regex::new(r"\$\(([A-Z_]+)\)").unwrap();

    for cap in re.captures_iter(content) {
        let var_name = &cap[1];
        let var_value = env::var(var_name)
            .with_context(|| format!("Environment variable {} not set", var_name))?;
        result = result.replace(&format!("$({})", var_name), &var_value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_env_vars() {
        env::set_var("TEST_USER", "testuser");
        env::set_var("TEST_PASSWORD", "testpass");

        let input = "postgresql://$(TEST_USER):$(TEST_PASSWORD)@localhost";
        let result = substitute_env_vars(input).unwrap();

        assert_eq!(result, "postgresql://testuser:testpass@localhost");
    }
}
