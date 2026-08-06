use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub app: AppConfig,
    #[serde(default)]
    pub smtp: Option<SmtpConfig>,
    #[serde(default)]
    pub dev_user: Option<DevUser>,
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
pub struct AppConfig {
    pub base_url: String,
    pub admin_emails: Vec<String>,
    pub ical_token: String,
    /// Optional map from email (lowercase) to username, for accounts whose
    /// email local-part is not a nice name (e.g. firstnamelastname@icloud.com).
    #[serde(default)]
    pub display_names: std::collections::HashMap<String, String>,
    /// Usernames of the students tracked by the korschema module. Progress
    /// rows are keyed by these names; other usernames are rejected.
    #[serde(default)]
    pub students: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub notify_email: String,
}

/// Local development only: injected as the authenticated user when the
/// oauth2-proxy headers are absent. Must never be set in the production config.
#[derive(Debug, Deserialize, Clone)]
pub struct DevUser {
    pub username: String,
    pub email: String,
}

impl Config {
    /// Load configuration from file with environment variable substitution
    pub fn load() -> Result<Self> {
        let config_path =
            env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.yaml".to_string());

        tracing::info!("Loading configuration from: {}", config_path);

        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path))?;

        let config_content = substitute_env_vars(&config_content)?;

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

    pub fn is_admin(&self, email: Option<&str>) -> bool {
        match email {
            Some(email) => self
                .app
                .admin_emails
                .iter()
                .any(|a| a.eq_ignore_ascii_case(email)),
            None => false,
        }
    }

    /// Human-readable username. oauth2-proxy fills X-Auth-Request-User from
    /// the OIDC `sub` claim, which in Authelia is an opaque UUID - so prefer,
    /// in order: the configured display_names entry for the email, the email
    /// local-part (erik@k12n.com -> erik), and finally the raw header value.
    pub fn username_for(&self, header_user: &str, email: Option<&str>) -> String {
        if let Some(email) = email {
            if let Some(name) = self.app.display_names.get(&email.to_lowercase()) {
                return name.clone();
            }
            if let Some(local) = email.split('@').next().filter(|s| !s.is_empty()) {
                return local.to_string();
            }
        }
        header_user.to_string()
    }
}

/// Substitute environment variables in format $(VAR_NAME)
fn substitute_env_vars(content: &str) -> Result<String> {
    substitute_vars(content, |name| env::var(name).ok())
}

fn substitute_vars(content: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<String> {
    let mut result = content.to_string();
    let re = regex::Regex::new(r"\$\(([A-Z_]+)\)").unwrap();

    for cap in re.captures_iter(content) {
        let var_name = &cap[1];
        let var_value = lookup(var_name)
            .with_context(|| format!("Environment variable {} not set", var_name))?;
        result = result.replace(&format!("$({})", var_name), &var_value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(admin_emails: Vec<String>) -> Config {
        Config {
            database: DatabaseConfig {
                url: "postgresql://localhost".into(),
                max_connections: 5,
                min_connections: 1,
            },
            api: ApiConfig {
                host: "127.0.0.1".into(),
                port: 8080,
            },
            app: AppConfig {
                base_url: "https://gasa.k12n.com".into(),
                admin_emails,
                ical_token: "secret".into(),
                display_names: std::collections::HashMap::new(),
                students: vec![],
            },
            smtp: None,
            dev_user: None,
        }
    }

    #[test]
    fn test_substitute_vars() {
        // Injected lookup instead of process env: thread-safe under the
        // multi-threaded test runner
        let lookup = |name: &str| match name {
            "TEST_USER" => Some("testuser".to_string()),
            "TEST_PASSWORD" => Some("testpass".to_string()),
            _ => None,
        };

        let input = "postgresql://$(TEST_USER):$(TEST_PASSWORD)@localhost";
        let result = substitute_vars(input, lookup).unwrap();

        assert_eq!(result, "postgresql://testuser:testpass@localhost");
    }

    #[test]
    fn test_substitute_vars_missing_var_errors() {
        let result = substitute_vars("$(MISSING_VAR)", |_| None);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_admin_case_insensitive() {
        let config = test_config(vec!["Admin@Example.com".into()]);
        assert!(config.is_admin(Some("admin@example.com")));
        assert!(config.is_admin(Some("ADMIN@EXAMPLE.COM")));
    }

    #[test]
    fn test_is_admin_rejects_other_and_missing_email() {
        let config = test_config(vec!["admin@example.com".into()]);
        assert!(!config.is_admin(Some("kid@example.com")));
        assert!(!config.is_admin(None));
    }

    #[test]
    fn test_username_for_prefers_display_names_map() {
        let mut config = test_config(vec![]);
        config
            .app
            .display_names
            .insert("ludvigkristoffersson@icloud.com".into(), "ludvig".into());
        assert_eq!(
            config.username_for("some-uuid", Some("LudvigKristoffersson@icloud.com")),
            "ludvig"
        );
    }

    #[test]
    fn test_username_for_falls_back_to_email_local_part() {
        let config = test_config(vec![]);
        assert_eq!(
            config.username_for(
                "18c33579-7073-4b53-840d-ae8ab2adc9aa",
                Some("erik@k12n.com")
            ),
            "erik"
        );
    }

    #[test]
    fn test_username_for_falls_back_to_header_user() {
        let config = test_config(vec![]);
        assert_eq!(config.username_for("someuser", None), "someuser");
        assert_eq!(config.username_for("someuser", Some("@broken")), "someuser");
    }
}
