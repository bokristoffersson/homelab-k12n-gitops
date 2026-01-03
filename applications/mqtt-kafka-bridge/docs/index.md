# MQTT-Kafka Bridge

## Overview

The MQTT-Kafka Bridge is a critical component in the homelab data pipeline that connects MQTT IoT devices to the Kafka streaming platform.

## Purpose

- **Subscribes** to MQTT topics from IoT devices (heat pumps, energy meters, temperature sensors)
- **Transforms** MQTT messages into Kafka messages
- **Publishes** to Redpanda (Kafka) topics for downstream processing

## Architecture

```
IoT Devices → Mosquitto MQTT Broker → MQTT-Kafka Bridge → Redpanda (Kafka)
```

## Key Features

- **Topic Mapping**: Maps MQTT topics to Kafka topics
- **Message Transformation**: Converts MQTT payloads to structured Kafka messages
- **Reliability**: Ensures messages are not lost during the bridge process
- **Monitoring**: Exposes metrics for observability

## Data Flow

1. IoT devices publish data to Mosquitto MQTT broker
2. MQTT-Kafka Bridge subscribes to configured MQTT topics
3. Messages are transformed and enriched with metadata
4. Transformed messages are published to Redpanda Kafka topics
5. Downstream consumers (TimescaleDB sink, WebSocket server, etc.) process the data

## Related Components

- **Mosquitto**: MQTT broker receiving IoT device data
- **Redpanda**: Kafka-compatible streaming platform
- **TimescaleDB Sink**: Consumes Kafka data and stores in database
- **Energy WebSocket**: Streams real-time data to web clients
