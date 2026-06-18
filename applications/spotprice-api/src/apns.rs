//! Apple Push Notification service sender.
//!
//! Token-based auth (.p8 key, ES256 provider JWT) over HTTP/2 to APNs. Uses
//! `reqwest` (rustls) and `jsonwebtoken` directly — both already in the tree on
//! the modern TLS stack — instead of a dedicated APNs crate, to avoid an extra
//! (and older-rustls) dependency.

use crate::config::ApnsConfig;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::Serialize;

const APNS_PRODUCTION: &str = "https://api.push.apple.com";
const APNS_SANDBOX: &str = "https://api.sandbox.push.apple.com";

/// Sends APNs notifications. One instance is shared for both environments;
/// the target host is chosen per device token's registered environment.
pub struct ApnsSender {
    http: Client,
    encoding_key: EncodingKey,
    key_id: String,
    team_id: String,
    bundle_id: String,
}

/// Provider authentication token claims (Apple uses `iss` = team id, `iat` = now).
#[derive(Serialize)]
struct ProviderClaims<'a> {
    iss: &'a str,
    iat: i64,
}

impl ApnsSender {
    pub fn from_config(cfg: &ApnsConfig) -> anyhow::Result<Self> {
        let key_pem = std::fs::read(&cfg.key_path)?;
        let encoding_key = EncodingKey::from_ec_pem(&key_pem)?;
        // APNs only speaks HTTP/2; force it rather than relying on ALPN.
        let http = Client::builder().http2_prior_knowledge().build()?;
        Ok(Self {
            http,
            encoding_key,
            key_id: cfg.key_id.clone(),
            team_id: cfg.team_id.clone(),
            bundle_id: cfg.bundle_id.clone(),
        })
    }

    /// Build a short-lived ES256 provider token signed with the .p8 key.
    fn provider_token(&self) -> anyhow::Result<String> {
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());
        let claims = ProviderClaims {
            iss: &self.team_id,
            iat: chrono::Utc::now().timestamp(),
        };
        Ok(encode(&header, &claims, &self.encoding_key)?)
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
    ) -> anyhow::Result<()> {
        let base = match environment {
            "sandbox" => APNS_SANDBOX,
            _ => APNS_PRODUCTION,
        };
        let url = format!("{base}/3/device/{device_token}");

        let payload = serde_json::json!({
            "aps": {
                "alert": { "title": title, "body": body },
                "sound": "default",
            },
            "spotprice": custom,
        });

        let token = self.provider_token()?;
        let response = self
            .http
            .post(&url)
            .header("authorization", format!("bearer {token}"))
            .header("apns-topic", &self.bundle_id)
            .header("apns-push-type", "alert")
            .header("apns-priority", "10")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let detail = response.text().await.unwrap_or_default();
            anyhow::bail!("APNs returned {}: {}", status, detail);
        }
        Ok(())
    }
}
