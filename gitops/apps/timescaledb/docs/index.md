# TimescaleDB

## Overview

TimescaleDB is a PostgreSQL-based time-series database that stores all IoT sensor data for the homelab. It provides fast queries over historical data and powers dashboards and analytics.

## Purpose

- **Store** time-series data from Kafka topics
- **Query** historical data for dashboards and analysis
- **Aggregate** data for efficient storage and querying
- **Backup** critical data with automated S3 backups

## Architecture

```
Redpanda (Kafka) → Redpanda Connect Sink → TimescaleDB → Grafana/API
```

## Components

### TimescaleDB Server

- **Image**: `timescale/timescaledb:latest-pg16`
- **Storage**: 20Gi persistent volume
- **Port**: 5432
- **Namespace**: timescaledb

### Redpanda Connect Sink

Streams data from Kafka topics into TimescaleDB tables:

- **Consumer Groups**:
  - `timescaledb-energy`: Energy consumption data
  - `timescaledb-heatpump`: Heat pump telemetry
  - `timescaledb-temperature`: Temperature readings

## Data Sources

| Kafka Topic | Table | Update Frequency |
|-------------|-------|------------------|
| `homelab.energy` | `energy_readings` | Real-time |
| `homelab.heatpump` | `heatpump_status` | Every 30s |
| `homelab.temperature` | `temperature_readings` | Every 5m |

## Features

### Hypertables

All time-series tables are converted to TimescaleDB hypertables for optimal performance:

```sql
SELECT create_hypertable('energy_readings', 'timestamp');
```

### Continuous Aggregates

Pre-computed aggregates for fast dashboard queries:

- `energy_hourly`: Hourly energy consumption
- `heatpump_daily`: Daily heat pump statistics
- `temperature_stats`: Temperature statistics per room

### Data Retention

Automated data retention policies:

- Raw data: 90 days
- Hourly aggregates: 2 years
- Daily aggregates: 5 years

## Monitoring

- **Grafana**: Visualizes metrics from TimescaleDB
- **Homelab API**: Serves data to web applications
- **Backups**: Daily automated backups to S3

## Related Components

- **Redpanda Sink (TimescaleDB)**: Ingests data from Kafka
- **Homelab API**: Queries data for web applications
- **Grafana**: Visualization and alerting
