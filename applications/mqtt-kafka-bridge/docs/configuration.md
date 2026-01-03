# Configuration

## MQTT Connection

The bridge connects to the Mosquitto MQTT broker running in the cluster:

```yaml
mqtt:
  broker: mosquitto.mosquitto.svc.cluster.local:1883
  client_id: mqtt-kafka-bridge
```

## Kafka Connection

Connects to the Redpanda cluster:

```yaml
kafka:
  brokers:
    - redpanda-v2.redpanda-v2.svc.cluster.local:9092
  topics:
    energy: homelab.energy
    heatpump: homelab.heatpump
    temperature: homelab.temperature
```

## Topic Mapping

MQTT topics are mapped to Kafka topics:

| MQTT Topic | Kafka Topic | Description |
|-----------|-------------|-------------|
| `homelab/energy/#` | `homelab.energy` | Energy consumption data |
| `homelab/heatpump/#` | `homelab.heatpump` | Heat pump telemetry |
| `homelab/temperature/#` | `homelab.temperature` | Temperature sensor readings |

## Environment Variables

- `MQTT_BROKER`: MQTT broker address
- `KAFKA_BROKERS`: Comma-separated list of Kafka brokers
- `LOG_LEVEL`: Logging level (info, debug, error)
