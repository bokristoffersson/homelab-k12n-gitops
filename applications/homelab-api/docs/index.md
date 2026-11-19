# Homelab API

Read-only REST API serving telemetry data from TimescaleDB.

## Overview

Homelab API is a Rust/Axum-based REST service that provides read-only access to time-series telemetry data stored in TimescaleDB. It serves data for energy consumption, heat pump status, and temperature sensors.

## Key Features

- **Read-only operations**: No write/update/delete operations allowed
- **JWT authentication**: Token-based auth via Authelia OIDC
- **Time-series data**: Optimized queries for TimescaleDB hypertables
- **Low latency**: Rust/Axum for high performance
- **CORS enabled**: Configured for frontend access

## Technology Stack

- **Language**: Rust 1.83
- **Framework**: Axum (async web framework)
- **Database**: TimescaleDB/PostgreSQL via sqlx
- **Authentication**: JWT validation (Authelia JWKS)
- **Container**: Multi-stage Docker build with ARM64 support

## Data Sources

The API queries three main data types:

1. **Energy**: Real-time power consumption from Shelly EM sensor
2. **Heat pump**: Status and performance metrics
3. **Temperature**: Environmental sensor readings (Shelly H&T)

All data originates from MQTT sensors, flows through Redpanda (Kafka), and is written to TimescaleDB by the Redpanda Connect sink.

## API Endpoints

Base URL: `https://homelab.k12n.com`

| Group | Endpoints |
|---|---|
| Energy | `latest`, `hourly-total`, `history`, `daily-summary`, `monthly-summary`, `yearly-summary` under `/api/v1/energy/` |
| Heat pump | `latest`, `daily-summary` under `/api/v1/heatpump/` |
| Temperature | `latest`, `all-latest`, `history` under `/api/v1/temperature/` |

Every endpoint needs an Authelia access token in `Authorization: Bearer <token>` with
the matching `read:*` scope. See the [API Reference](api.md).

## Related Components

- **Frontend**: [heatpump-web](../heatpump-web) - React SPA consuming this API
- **Database**: [timescaledb](../../gitops/apps/timescaledb) - Time-series data store
- **Data Pipeline**: [mqtt-kafka-bridge](../mqtt-kafka-bridge) → [Redpanda Connect sink](../../gitops/apps/timescaledb)
