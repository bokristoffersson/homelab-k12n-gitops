mod config;
mod db;
mod kafka;
mod outbox;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting homelab-settings-outbox-processor");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");
    info!("MQTT broker: {}", config.mqtt_broker);
    info!("Kafka brokers: {}", config.kafka_brokers);
    info!("Kafka topic: {}", config.kafka_topic);
    info!("Kafka group: {}", config.kafka_group_id);
    info!("Poll interval: {}s", config.poll_interval_secs);

    // Connect to database
    let pool = db::connect(&config.database_url).await?;
    info!("Connected to database successfully");

    // Connect to MQTT broker
    // Parse broker address (format: "host:port")
    let parts: Vec<&str> = config.mqtt_broker.split(':').collect();
    let host = parts
        .get(0)
        .unwrap_or(&"mosquitto.mosquitto.svc.cluster.local")
        .to_string();
    let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1883);

    // Use hostname as unique client ID to prevent duplicate connections
    let client_id = std::env::var("HOSTNAME")
        .unwrap_or_else(|_| format!("outbox-processor-{}", std::process::id()));
    info!("MQTT client ID: {}", client_id);

    let mut mqtt_options = MqttOptions::new(client_id, host, port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));
    mqtt_options.set_clean_session(true);

    // Set MQTT credentials if provided
    info!("MQTT username from config: {:?}", config.mqtt_username);
    info!(
        "MQTT password from config: {:?}",
        config
            .mqtt_password
            .as_ref()
            .map(|p| format!("{}***", &p[..3.min(p.len())]))
    );

    if let (Some(username), Some(password)) = (&config.mqtt_username, &config.mqtt_password) {
        info!("Setting MQTT credentials for user: {}", username);
        mqtt_options.set_credentials(username, password);
        info!("MQTT authentication enabled");
    } else {
        info!("MQTT authentication disabled (anonymous)");
    }

    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    info!("MQTT client created");

    // Spawn MQTT event loop handler
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    error!("MQTT event loop error: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    // Spawn Kafka confirmation listener
    let pool_clone = pool.clone();
    let kafka_brokers = config.kafka_brokers.clone();
    let kafka_topic = config.kafka_topic.clone();
    let kafka_group_id = config.kafka_group_id.clone();
    tokio::spawn(async move {
        kafka::start_confirmation_listener(pool_clone, kafka_brokers, kafka_topic, kafka_group_id)
            .await;
    });
    info!("Kafka confirmation listener spawned");

    // Main processing loop
    loop {
        match process_pending_entries(&pool, &mqtt_client).await {
            Ok(processed) => {
                if processed > 0 {
                    info!("Processed {} pending outbox entries", processed);
                }
            }
            Err(e) => {
                error!("Error processing outbox entries: {}", e);
            }
        }

        sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

/// Map database field names to ThermIQ "d" parameter names
fn map_field_to_thermiq_param(field_name: &str) -> Option<String> {
    match field_name {
        "indoor_target_temp" => Some("d50".to_string()),
        "mode" => Some("d51".to_string()),
        "curve" => Some("d52".to_string()),
        "curve_min" => Some("d53".to_string()),
        "curve_max" => Some("d54".to_string()),
        "curve_plus_5" => Some("d55".to_string()),
        "curve_zero" => Some("d56".to_string()),
        "curve_minus_5" => Some("d57".to_string()),
        "heatstop" => Some("d58".to_string()),
        "integral_setting" => Some("d73".to_string()),
        _ => None,
    }
}

/// Publish a power plug command to Tasmota via MQTT
/// Topic format: cmnd/{plug_id}/POWER
/// Payload: ON or OFF
async fn publish_plug_command(
    mqtt_client: &AsyncClient,
    entry: &outbox::OutboxEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload_obj = entry
        .payload
        .as_object()
        .ok_or("Payload is not a JSON object")?;

    // Get plug_id from payload (or fall back to aggregate_id)
    let plug_id = payload_obj
        .get("plug_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&entry.aggregate_id);

    // Get action from payload
    let action = payload_obj
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'action' field in payload")?;

    // Validate action
    if action != "ON" && action != "OFF" {
        return Err(format!("Invalid action '{}', expected 'ON' or 'OFF'", action).into());
    }

    // Build Tasmota MQTT topic: cmnd/{plug_id}/POWER
    let topic = format!("cmnd/{}/POWER", plug_id);

    // Log source information if available
    let source = payload_obj
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let schedule_id = payload_obj.get("schedule_id").and_then(|v| v.as_i64());

    if let Some(sid) = schedule_id {
        info!(
            "Publishing plug command (source={}, schedule_id={}) to topic '{}': {}",
            source, sid, topic, action
        );
    } else {
        info!(
            "Publishing plug command (source={}) to topic '{}': {}",
            source, topic, action
        );
    }

    // Publish to MQTT
    mqtt_client
        .publish(&topic, QoS::AtLeastOnce, false, action.as_bytes())
        .await?;

    info!("Published plug command to {}: {}", topic, action);
    Ok(())
}

/// Publish a heatpump setting command to ThermIQ via MQTT
async fn publish_heatpump_command(
    mqtt_client: &AsyncClient,
    entry: &outbox::OutboxEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    let topic = "thermiq_heatpump/write";

    let payload_obj = entry
        .payload
        .as_object()
        .ok_or("Payload is not a JSON object")?;

    // Convert each field to ThermIQ "d" parameter and publish separately
    for (field_name, field_value) in payload_obj {
        if let Some(thermiq_param) = map_field_to_thermiq_param(field_name) {
            // Build single-field payload: {"d50": 21}
            let thermiq_payload = serde_json::json!({
                thermiq_param: field_value
            });
            let payload_str = thermiq_payload.to_string();

            info!(
                "Publishing outbox entry {} field '{}' to topic '{}': {}",
                entry.id, field_name, topic, payload_str
            );

            // Publish to MQTT
            mqtt_client
                .publish(topic, QoS::AtLeastOnce, false, payload_str.as_bytes())
                .await?;

            info!("Published field '{}'", field_name);
        } else {
            warn!("Unknown field '{}' in payload, skipping", field_name);
        }
    }

    Ok(())
}

async fn process_pending_entries(
    pool: &sqlx::PgPool,
    mqtt_client: &AsyncClient,
) -> Result<usize, Box<dyn std::error::Error>> {
    let entries = outbox::get_pending_entries(pool, 10).await?;

    if entries.is_empty() {
        return Ok(0);
    }

    info!("Found {} pending entries to process", entries.len());

    for entry in &entries {
        // Route based on aggregate_type
        let publish_result = match entry.aggregate_type.as_str() {
            "power_plug" => {
                info!(
                    "Processing power plug command (id={}, event_type={})",
                    entry.id, entry.event_type
                );
                publish_plug_command(mqtt_client, entry).await
            }
            "heatpump_setting" => {
                info!(
                    "Processing heatpump setting command (id={}, event_type={})",
                    entry.id, entry.event_type
                );
                publish_heatpump_command(mqtt_client, entry).await
            }
            unknown => {
                warn!(
                    "Unknown aggregate_type '{}' for entry {}, skipping",
                    unknown, entry.id
                );
                // Mark as failed since we don't know how to process it
                let _ = outbox::mark_failed(
                    pool,
                    entry.id,
                    &format!("Unknown aggregate_type: {}", unknown),
                )
                .await;
                continue;
            }
        };

        match publish_result {
            Ok(_) => {
                // Mark as published
                match outbox::mark_published(pool, entry.id).await {
                    Ok(_) => {
                        info!("Marked outbox entry {} as published", entry.id);
                    }
                    Err(e) => {
                        error!("Failed to mark entry {} as published: {}", entry.id, e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to publish entry {} (aggregate_type={}): {}",
                    entry.id, entry.aggregate_type, e
                );

                // Handle retry logic
                if entry.retry_count + 1 >= entry.max_retries {
                    warn!(
                        "Entry {} exceeded max retries ({}), marking as failed",
                        entry.id, entry.max_retries
                    );
                    match outbox::mark_failed(pool, entry.id, &e.to_string()).await {
                        Ok(_) => {
                            error!("Marked outbox entry {} as failed", entry.id);
                        }
                        Err(db_err) => {
                            error!("Failed to mark entry {} as failed: {}", entry.id, db_err);
                        }
                    }
                } else {
                    // Increment retry count
                    match outbox::increment_retry(pool, entry.id, &e.to_string()).await {
                        Ok(_) => {
                            warn!(
                                "Incremented retry count for entry {} ({}/{})",
                                entry.id,
                                entry.retry_count + 1,
                                entry.max_retries
                            );
                        }
                        Err(db_err) => {
                            error!(
                                "Failed to increment retry for entry {}: {}",
                                entry.id, db_err
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(entries.len())
}
