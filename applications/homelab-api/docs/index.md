# Homelab API

Read-only REST API serving telemetry data from TimescaleDB.

## Overview

Homelab API is a Rust/Axum-based REST service that provides read-only access to time-series telemetry data stored in TimescaleDB. It serves data for energy consumption, heat pump status, and temperature sensors.

## Key Features

- **Read-only operations**: No write/update/delete operations allowed
- **JWT authentication**: Token-based auth via Authentik OIDC
- **Time-series data**: Optimized queries for TimescaleDB hypertables
- **Low latency**: Rust/Axum for high performance
- **CORS enabled**: Configured for frontend access

## Technology Stack

- **Language**: Rust 1.83
- **Framework**: Axum (async web framework)
- **Database**: TimescaleDB/PostgreSQL via sqlx
- **Authentication**: JWT validation (Authentik JWKS)
- **Container**: Multi-stage Docker build with ARM64 support

## Data Sources

The API queries three main data types:

1. **Energy**: Real-time power consumption from Shelly EM sensor
2. **Heat pump**: Status and performance metrics
3. **Temperature**: Environmental sensor readings (Shelly H&T)

All data originates from MQTT sensors, flows through Redpanda (Kafka), and is persisted by redpanda-sink to TimescaleDB.

## API Endpoints

Base URL: `https://homelab-api.k12n.com`

- `GET /api/v1/energy/latest` - Latest energy reading
- `GET /api/v1/energy/24h` - 24-hour energy history
- `GET /api/v1/heatpump/latest` - Current heat pump status
- `GET /api/v1/heatpump/24h` - 24-hour heat pump history
- `GET /api/v1/temperature/latest` - Latest temperature readings
- `GET /api/v1/temperature/24h` - 24-hour temperature history

All endpoints require valid JWT token in `Authorization: Bearer <token>` header.

## Related Components

- **Frontend**: [heatpump-web](../heatpump-web) - React SPA consuming this API
- **Database**: [timescaledb](../../gitops/apps/base/timescaledb) - Time-series data store
- **Data Pipeline**: [mqtt-kafka-bridge](../mqtt-kafka-bridge) → [redpanda-sink](../../gitops/apps/base/timescaledb)
