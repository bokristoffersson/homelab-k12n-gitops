use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub poll_interval_secs: u64,
    pub mqtt_broker: String,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub kafka_group_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            poll_interval_secs: env::var("POLL_INTERVAL_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
            mqtt_broker: env::var("MQTT_BROKER")
                .unwrap_or_else(|_| "mosquitto.mosquitto.svc.cluster.local:1883".to_string()),
            mqtt_username: env::var("MQTT_USERNAME").ok(),
            mqtt_password: env::var("MQTT_PASSWORD").ok(),
            kafka_brokers: env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "redpanda-v2.redpanda-v2.svc.cluster.local:9092".to_string()),
            kafka_topic: env::var("KAFKA_TOPIC")
                .unwrap_or_else(|_| "homelab-heatpump-telemetry".to_string()),
            kafka_group_id: env::var("KAFKA_GROUP_ID")
                .unwrap_or_else(|_| "heatpump-settings-outbox-processor".to_string()),
        })
    }
}
