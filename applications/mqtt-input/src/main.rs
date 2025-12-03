mod config;
mod error;
mod ingest;
mod mapping;
mod mqtt;
mod redpanda;

use config::Config;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    let cfg_path =
        std::env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.example.yaml".into());
    let cfg = Config::load(&cfg_path)?;
    info!("loaded config; pipelines: {}", cfg.pipelines.len());

    let producer = redpanda::create_producer(&cfg.redpanda.brokers).await?;
    info!(
        brokers = %cfg.redpanda.brokers,
        "connected to Redpanda"
    );

    let keep_alive = cfg.mqtt.keep_alive_secs.unwrap_or(30);
    let clean = cfg.mqtt.clean_session.unwrap_or(true);
    let ca_file = cfg.mqtt.tls.as_ref().map(|t| t.ca_file.clone());
    let opts = mqtt::build_options(
        &cfg.mqtt.host,
        cfg.mqtt.port,
        &cfg.mqtt.username,
        &cfg.mqtt.password,
        keep_alive,
        clean,
        &ca_file,
    )?;
    let (client, mut eventloop) = mqtt::new(opts);
    for p in &cfg.pipelines {
        client.subscribe(p.topic.clone(), mqtt::qos(p.qos)).await?;
    }
    info!("subscribed to {} pipeline topic(s)", cfg.pipelines.len());

    let ingestor = ingest::Ingestor::new(producer);

    let sig = tokio::signal::ctrl_c();
    tokio::pin!(sig);
    loop {
        tokio::select! {
            biased;
            _ = &mut sig => {
                info!("shutdown requested");
                break;
            }
            res = mqtt::next_publish(&mut eventloop) => {
                match res {
                    Ok(Some(msg)) => {
                        let topic_bytes = msg.topic;
                        let payload = msg.payload;
                        let topic = match std::str::from_utf8(&topic_bytes) {
                            Ok(s) => s.to_string(),
                            Err(_) => {
                                warn!(?topic_bytes, "non-utf8 topic; skipping message");
                                continue;
                            }
                        };
                        if let Err(e) = ingestor.handle_message(&cfg.pipelines, &topic, payload.as_ref()).await {
                            warn!(topic=%topic, error=%e, "processing failed for incoming message");
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        warn!("mqtt error: {e}; reconnecting after short delay");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }
    }

    Ok(())
}
