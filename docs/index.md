# Homelab Streaming Architecture

## Overview

This homelab implements a complete IoT data streaming pipeline using open-source technologies to collect, process, store, and visualize sensor data from various devices.

## High-Level Architecture

```
┌─────────────┐     ┌──────────┐     ┌────────────┐     ┌─────────────┐
│ IoT Devices │────▶│ Mosquitto│────▶│ MQTT-Kafka │────▶│  Redpanda   │
│  (MQTT)     │     │  Broker  │     │   Bridge   │     │   (Kafka)   │
└─────────────┘     └──────────┘     └────────────┘     └─────────────┘
                                                                │
                         ┌──────────────────────────────────────┴────────┐
                         │                                                │
                         ▼                                                ▼
                  ┌─────────────┐                              ┌─────────────────┐
                  │ TimescaleDB │                              │  Energy WebSocket│
                  │   (Sink)    │                              │     Server       │
                  └─────────────┘                              └─────────────────┘
                         │                                                │
                         ▼                                                ▼
                  ┌─────────────┐                              ┌─────────────────┐
                  │   Grafana   │                              │   Heatpump Web  │
                  │  (Dashboards)│                              │    Dashboard    │
                  └─────────────┘                              └─────────────────┘
```

## Components

### Data Ingestion

- **IoT Devices**: Heat pumps, energy meters, temperature sensors
- **Mosquitto**: MQTT broker for device communication
- **MQTT-Kafka Bridge**: Converts MQTT messages to Kafka events

### Streaming Platform

- **Redpanda**: Kafka-compatible streaming platform
- **Topics**: homelab.energy, homelab.heatpump, homelab.temperature
- **Consumer Groups**: Multiple consumers for different purposes

### Data Storage

- **TimescaleDB**: PostgreSQL-based time-series database
- **Redpanda Sink**: Streams Kafka data into TimescaleDB
- **Retention**: 90 days raw data, 2 years aggregates

### Data Processing

- **Redpanda Connect**: Stream processing and transformations
- **Continuous Aggregates**: Pre-computed metrics in TimescaleDB
- **Real-time Processing**: WebSocket server for live data

### Visualization & APIs

- **Grafana**: Dashboards and alerting
- **Homelab API**: REST API for web applications
- **Energy WebSocket**: Real-time data streaming to browsers
- **Heatpump Web**: SPA dashboard for heat pump monitoring

## Key Features

### Reliability

- **At-least-once delivery**: Messages are not lost
- **Consumer groups**: Multiple consumers can process same data
- **Dead-letter queues**: Failed messages are retained
- **Automated backups**: Daily TimescaleDB backups to S3

### Scalability

- **Horizontal scaling**: Add more consumers as needed
- **Partitioning**: Kafka topics can be partitioned
- **Continuous aggregates**: Pre-computed for fast queries
- **Efficient storage**: TimescaleDB compression

### Observability

- **Metrics**: Prometheus metrics from all components
- **Logging**: Centralized logging with kubectl
- **Tracing**: Kafka consumer group lag monitoring
- **Dashboards**: Grafana visualization of system health

## Technologies

| Component | Technology | Version |
|-----------|-----------|---------|
| Message Broker | Mosquitto | 2.x |
| Streaming Platform | Redpanda | latest |
| Stream Processing | Redpanda Connect | latest |
| Database | TimescaleDB | 2.x (PG16) |
| Visualization | Grafana | 11.x |
| Container Orchestration | Kubernetes (K3s) | 1.33 |
| GitOps | Flux CD | 2.x |

## Getting Started

1. **View Services**: Navigate to Backstage catalog
2. **Check Health**: View Kubernetes tab on each service
3. **Monitor Data**: Check Kafka consumer group lag
4. **Query Data**: Use Grafana or Homelab API
5. **View Docs**: Each component has detailed TechDocs

## Related Documentation

- [Data Flow](data-flow.md): Detailed data pipeline explanation
- [Kafka Topics](kafka-topics.md): Topic naming and configuration
- [Monitoring](monitoring.md): How to monitor the system
