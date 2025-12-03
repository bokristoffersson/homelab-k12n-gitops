use crate::error::AppError;
use std::time::Duration;
use uuid::Uuid;

// Use the MQTT v5 API surface only
use rumqttc::v5 as mqtt5;
use rumqttc::Transport;

// Re-export types so the rest of the code can use these names
pub type MqttOptions = mqtt5::MqttOptions;
pub type AsyncClient = mqtt5::AsyncClient;
pub type EventLoop = mqtt5::EventLoop;
pub type Incoming = mqtt5::Incoming;
pub type V5Publish = mqtt5::mqttbytes::v5::Publish;

pub fn build_options(
    host: &str,
    port: u16,
    username: &Option<String>,
    password: &Option<String>,
    keep_alive_secs: u64,
    clean_start: bool,
    _ca_file: &Option<String>,
) -> Result<MqttOptions, AppError> {
    let client_id = format!("mqtt-input-{}", Uuid::new_v4());
    // Using v5::MqttOptions selects MQTT 5
    let mut opts = MqttOptions::new(client_id, host, port);
    opts.set_keep_alive(Duration::from_secs(keep_alive_secs));
    opts.set_clean_start(clean_start);
    if let (Some(u), Some(p)) = (username, password) {
        opts.set_credentials(u.clone(), p.clone());
    }
    if port == 8883 {
        opts.set_transport(Transport::tls_with_default_config());
    }
    Ok(opts)
}

pub fn new(options: MqttOptions) -> (AsyncClient, EventLoop) {
    mqtt5::AsyncClient::new(options, 50)
}

// Return the v5 QoS type explicitly
pub fn qos(v: u8) -> mqtt5::mqttbytes::QoS {
    match v {
        2 => mqtt5::mqttbytes::QoS::ExactlyOnce,
        0 => mqtt5::mqttbytes::QoS::AtMostOnce,
        _ => mqtt5::mqttbytes::QoS::AtLeastOnce,
    }
}

pub async fn next_publish(eventloop: &mut EventLoop) -> Result<Option<V5Publish>, AppError> {
    loop {
        match eventloop.poll().await {
            Ok(mqtt5::Event::Incoming(mqtt5::Incoming::Publish(p))) => return Ok(Some(p)),
            Ok(_) => continue,
            Err(e) => return Err(AppError::Mqtt(e.to_string())),
        }
    }
}
