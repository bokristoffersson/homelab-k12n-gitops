use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub kafka: KafkaConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
    pub group_id: String,
    pub auto_offset_reset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

/// Single issuer configuration for multi-issuer support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerConfig {
    pub name: String,
    pub issuer: String,
    pub jwks_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    // Legacy HS256 configuration (optional)
    #[serde(default)]
    pub jwt_secret: Option<String>,

    // Multi-issuer JWKS configuration for RS256 validation from Authentik
    #[serde(default)]
    pub issuers: Option<Vec<IssuerConfig>>,

    // Legacy single-issuer configuration (for backwards compatibility)
    #[serde(default)]
    pub jwks_url: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
}

impl Config {
    /// Load configuration from a YAML file with environment variable substitution
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        // Expand environment variables in the format $(VAR_NAME)
        let expanded = expand_env_vars(&content);

        let config: Config = serde_yaml::from_str(&expanded)?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        if self.kafka.brokers.is_empty() {
            return Err(AppError::Config(
                "Kafka brokers cannot be empty".to_string(),
            ));
        }

        if self.kafka.topic.is_empty() {
            return Err(AppError::Config("Kafka topic cannot be empty".to_string()));
        }

        if self.kafka.group_id.is_empty() {
            return Err(AppError::Config(
                "Kafka group_id cannot be empty".to_string(),
            ));
        }

        if self.server.port == 0 {
            return Err(AppError::Config("Server port cannot be 0".to_string()));
        }

        // Validate that either legacy HS256, multi-issuer JWKS, or single-issuer JWKS is provided
        let has_legacy = self.auth.jwt_secret.is_some();
        let has_multi_issuer = self
            .auth
            .issuers
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let has_single_issuer = self.auth.jwks_url.is_some() && self.auth.issuer.is_some();

        if !has_legacy && !has_multi_issuer && !has_single_issuer {
            return Err(AppError::Config(
                "Either jwt_secret, issuers[], or (jwks_url and issuer) must be provided"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

/// Expand environment variables in the format $(VAR_NAME)
fn expand_env_vars(content: &str) -> String {
    let mut result = content.to_string();

    // Find all $(VAR_NAME) patterns
    let re = regex::Regex::new(r"\$\(([A-Z_][A-Z0-9_]*)\)").unwrap();

    for cap in re.captures_iter(content) {
        let full_match = &cap[0];
        let var_name = &cap[1];

        if let Ok(value) = std::env::var(var_name) {
            result = result.replace(full_match, &value);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars() {
        std::env::set_var("TEST_VAR", "test_value");

        let input = "secret: $(TEST_VAR)";
        let output = expand_env_vars(input);

        assert_eq!(output, "secret: test_value");

        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_expand_env_vars_not_found() {
        let input = "secret: $(NONEXISTENT_VAR)";
        let output = expand_env_vars(input);

        // Should leave it unchanged if not found
        assert_eq!(output, "secret: $(NONEXISTENT_VAR)");
    }
}
