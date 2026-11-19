# Homelab Streaming Architecture

## Overview

The homelab collects readings from the heat pump, the energy meter, temperature sensors
and smart plugs, stores them, and shows them in a web dashboard. Commands to the heat
pump and the plugs travel the other way.

## High-Level Architecture

```
Devices ──► Mosquitto ──► mqtt-kafka-bridge ──► Redpanda ──┬──► sink ──► TimescaleDB ──► homelab-api ──┐
 (MQTT)                                                    │                                           ├──► heatpump-web
                                                           └──► energy-ws (WebSocket) ─────────────────┘

heatpump-web ──► homelab-settings-api ──► outbox ──► Mosquitto ──► devices
```

## Components

| Stage | Component | Role |
|---|---|---|
| Ingestion | Mosquitto | MQTT broker the devices publish to |
| Ingestion | mqtt-kafka-bridge | Redpanda Connect pipelines that move MQTT messages into Redpanda |
| Streaming | Redpanda | Single-node, Kafka-compatible log. Topics are created by an `rpk` Job |
| Storage | Redpanda Connect sink | Writes telemetry topics into TimescaleDB |
| Storage | TimescaleDB | Time-series database: 90 days of raw data plus continuous aggregates |
| Serving | homelab-api | Read-only REST API over TimescaleDB |
| Serving | energy-ws | Pushes live energy readings to WebSocket clients |
| Serving | homelab-settings-api | Changes heat pump settings and switches plugs through a transactional outbox |
| UI | heatpump-web | The dashboard at `homelab.k12n.com` |
| UI | Grafana | Infrastructure dashboards at `grafana.k12n.com` |

Each component has its own TechDocs page in Backstage.

## Properties

- **At-least-once delivery.** Consumers commit their offset after processing, so a
  restart replays rather than drops. Device commands are level-triggered (ON/OFF), which
  makes replays safe.
- **Decoupled consumers.** The database sink, the WebSocket server and the settings
  service read the same topics independently.
- **Not highly available.** Redpanda and the databases are single instances. Longhorn
  keeps two replicas of each volume, and the databases are backed up to S3 daily.

## Further reading

- [Data Flow](data-flow.md): the path of a reading and of a command
- [Kafka Topics](kafka-topics.md): topics, retention and consumer groups
- [Monitoring](monitoring.md): alerts, dashboards and how to investigate
