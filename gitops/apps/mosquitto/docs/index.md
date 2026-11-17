# Mosquitto MQTT Broker

MQTT message broker enabling communication between IoT devices and homelab services.

## Overview

Mosquitto is an open-source MQTT broker that handles pub/sub messaging for all IoT devices in the homelab. It provides a reliable, lightweight messaging protocol ideal for sensors and low-power devices.

## Key Features

- **MQTT 3.1.1/5.0 support**: Industry-standard protocol
- **Pub/Sub messaging**: Decoupled communication pattern
- **Topic-based routing**: Hierarchical message organization
- **Persistent sessions**: Retains subscriptions and messages
- **QoS levels**: Guaranteed delivery options (0, 1, 2)
- **Lightweight**: Minimal resource footprint

## Architecture

```
┌─────────────────┐
│  Shelly Devices │
│  - EM (energy)  │
│  - H&T (temp)   │
└────────┬────────┘
         │ MQTT pub
         ▼
┌─────────────────┐
│   Mosquitto     │
│   MQTT Broker   │
│   Port: 1883    │
└────────┬────────┘
         │ MQTT sub
         ▼
┌─────────────────┐
│mqtt-kafka-bridge│
│(Redpanda Connect│
└─────────────────┘
```

## Deployment

- **Namespace**: `mosquitto`
- **Port**: 1883 (MQTT), 9001 (WebSocket - not exposed)
- **Replicas**: 1 (single broker)
- **Storage**: Persistent volume for message retention
- **Resources**: 50m CPU, 64Mi memory

## Connected Devices

### Shelly EM (Energy Monitor)
- **Topic**: `shellypro3em-485519a6aa98/events/rpc`
- **Frequency**: Every 1-2 seconds
- **Payload**: JSON with power, voltage, current readings

### Shelly H&T Gen3 (Temperature/Humidity)
- **Topic**: `shellyhtg3-e4b32322a0f4/events/rpc`
- **Frequency**: On change (±0.5°C or ±5% humidity)
- **Forced update**: Every 2 hours
- **Payload**: JSON with temperature and humidity

## Data Flow

1. **IoT devices** publish sensor readings to specific topics
2. **Mosquitto** receives and stores messages
3. **mqtt-kafka-bridge** subscribes to all relevant topics
4. **Bridge** transforms and forwards messages to Redpanda (Kafka)
5. **Redpanda-sink** persists data to TimescaleDB

## Topic Structure

```
shellyem-<device-id>/events/rpc     # Energy monitor events
shellyhtg3-<device-id>/events/rpc   # Temperature sensor events
```

## MQTT Configuration

### QoS Settings
- **QoS 0**: At most once delivery (default for most sensors)
- **QoS 1**: At least once delivery (used for critical telemetry)
- **QoS 2**: Exactly once delivery (rarely needed)

### Persistence
- Messages are persisted to disk
- Subscriptions survive broker restarts
- Configurable retention period

## Monitoring

### Health Checks
- **Liveness probe**: TCP connection on port 1883
- **Readiness probe**: Successful MQTT CONNECT packet

### Metrics
- Connected clients count
- Messages received/sent per second
- Message queue size
- Uptime

## Security

- **Internal only**: Not exposed outside cluster
- **No TLS**: Internal traffic uses plain MQTT
- **No authentication**: Trusted cluster network
- **Future**: Consider adding auth for production hardening

## Troubleshooting

### Check Broker Status

```bash
kubectl get pods -n mosquitto
kubectl logs -n mosquitto -l app=mosquitto
```

### Test MQTT Connection

```bash
# Install mosquitto clients
apt-get install mosquitto-clients

# Port forward to broker
kubectl port-forward -n mosquitto svc/mosquitto 1883:1883

# Subscribe to all topics
mosquitto_sub -h localhost -t '#' -v

# Publish test message
mosquitto_pub -h localhost -t 'test/topic' -m 'Hello MQTT'
```

### Monitor Messages

```bash
# Watch specific device
mosquitto_sub -h localhost -t 'shellyhtg3-e4b32322a0f4/events/rpc' -v

# Watch all Shelly devices
mosquitto_sub -h localhost -t 'shelly+/events/rpc' -v
```

## Performance

- **Throughput**: 1000+ messages/second
- **Latency**: <10ms for message delivery
- **Connections**: Supports 100+ concurrent clients
- **Memory**: ~40MB RSS under normal load

## Related Components

- **Consumers**: [mqtt-kafka-bridge](../../../applications/mqtt-kafka-bridge) - Primary subscriber
- **Publishers**: Shelly IoT devices (EM, H&T sensors)
- **Downstream**: [Redpanda](../redpanda-v2) receives transformed messages
