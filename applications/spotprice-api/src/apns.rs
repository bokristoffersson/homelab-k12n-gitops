//! Apple Push Notification service sender (token-based auth, .p8 key).

use crate::config::ApnsConfig;
use a2::{
    Client, ClientConfig, DefaultNotificationBuilder, Endpoint, NotificationBuilder,
    NotificationOptions, PushType,
};

/// Holds one APNs client per environment so each device token can be delivered
/// to the endpoint (sandbox/production) it was registered against.
pub struct ApnsSender {
    production: Client,
    sandbox: Client,
    bundle_id: String,
}

impl ApnsSender {
    pub fn from_config(cfg: &ApnsConfig) -> anyhow::Result<Self> {
        let key = std::fs::read(&cfg.key_path)?;
        let production = Client::token(
            key.as_slice(),
            cfg.key_id.clone(),
            cfg.team_id.clone(),
            ClientConfig::new(Endpoint::Production),
        )?;
        let sandbox = Client::token(
            key.as_slice(),
            cfg.key_id.clone(),
            cfg.team_id.clone(),
            ClientConfig::new(Endpoint::Sandbox),
        )?;
        Ok(Self {
            production,
            sandbox,
            bundle_id: cfg.bundle_id.clone(),
        })
    }

    /// Send a visible alert notification to one device token.
    /// `custom` is attached under the `spotprice` key for the app to deep-link.
    pub async fn send(
        &self,
        device_token: &str,
        environment: &str,
        title: &str,
        body: &str,
        custom: &serde_json::Value,
    ) -> Result<(), a2::Error> {
        let builder = DefaultNotificationBuilder::new()
            .set_title(title)
            .set_body(body)
            .set_sound("default");

        let options = NotificationOptions {
            apns_topic: Some(self.bundle_id.as_str()),
            apns_push_type: Some(PushType::Alert),
            ..Default::default()
        };

        let mut payload = builder.build(device_token, options);
        payload.add_custom_data("spotprice", custom)?;

        let client = match environment {
            "sandbox" => &self.sandbox,
            _ => &self.production,
        };
        client.send(payload).await?;
        Ok(())
    }
}
