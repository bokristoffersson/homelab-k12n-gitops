# Kafka Topics

## Overview

All IoT data flows through Redpanda (Kafka-compatible) topics. This document describes topic naming, configuration, and usage.

## Topic Structure

### Naming Convention

```
<namespace>.<category>.<optional-subcategory>
```

**Examples:**
- `homelab.energy` - Energy consumption data
- `homelab.heatpump` - Heat pump telemetry
- `homelab.temperature` - Temperature sensors
- `homelab.heatpump.settings` - Heat pump configuration changes

## Active Topics

### homelab.energy

**Purpose**: Real-time energy consumption data from smart meters

**Schema:**
```json
{
  "timestamp": "2026-01-03T16:00:00Z",
  "device_id": "meter1",
  "total_energy_kwh": 12.45,
  "power_kw": 3.2,
  "voltage": 230,
  "current": 13.9
}
```

**Configuration:**
- **Partitions**: 3
- **Replication Factor**: 3
- **Retention**: 7 days
- **Compression**: snappy

**Producers:**
- mqtt-kafka-bridge

**Consumers:**
- timescaledb-energy (sink to database)
- energy-ws (WebSocket streaming)

### homelab.heatpump

**Purpose**: Heat pump status and telemetry

**Schema:**
```json
{
  "timestamp": "2026-01-03T16:00:00Z",
  "device_id": "heatpump1",
  "mode": "heating",
  "target_temp": 22.0,
  "actual_temp": 21.5,
  "compressor_running": true,
  "power_consumption_kw": 2.5
}
```

**Configuration:**
- **Partitions**: 1
- **Replication Factor**: 3
- **Retention**: 7 days
- **Compression**: snappy

**Producers:**
- mqtt-kafka-bridge

**Consumers:**
- timescaledb-heatpump (sink to database)

### homelab.temperature

**Purpose**: Temperature and humidity sensor readings

**Schema:**
```json
{
  "timestamp": "2026-01-03T16:00:00Z",
  "sensor_id": "temp_living_room",
  "room": "Living Room",
  "temperature": 22.3,
  "humidity": 45.2
}
```

**Configuration:**
- **Partitions**: 2
- **Replication Factor**: 3
- **Retention**: 7 days
- **Compression**: snappy

**Producers:**
- mqtt-kafka-bridge

**Consumers:**
- timescaledb-temperature (sink to database)

### homelab.heatpump.settings

**Purpose**: Heat pump configuration change requests

**Schema:**
```json
{
  "timestamp": "2026-01-03T16:00:00Z",
  "device_id": "heatpump1",
  "setting_type": "target_temperature",
  "value": 23.0,
  "user": "bo.kristoffersson"
}
```

**Configuration:**
- **Partitions**: 1
- **Replication Factor**: 3
- **Retention**: 30 days
- **Compression**: gzip

**Producers:**
- homelab-api

**Consumers:**
- heatpump-settings (processor)

## Topic Management

### List Topics

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic list
```

### Describe Topic

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic describe homelab.energy
```

### View Messages

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk topic consume homelab.energy --num 10 --format json
```

### Create New Topic

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk topic create homelab.new-topic \
    --partitions 3 \
    --replicas 3 \
    --topic-config retention.ms=604800000
```

## Partition Strategy

### Energy Topic (3 partitions)

Partitioned by device_id for parallel processing:

```
Partition 0: meter1
Partition 1: meter2
Partition 2: meter3
```

**Benefits:**
- Parallel consumption
- Order preserved per device
- Better throughput

### Heatpump Topic (1 partition)

Single partition for strict ordering:

**Reason:** Heat pump settings must be applied in order

## Retention Policies

| Topic | Retention | Reason |
|-------|-----------|--------|
| homelab.energy | 7 days | Data in TimescaleDB, Kafka for recent replay |
| homelab.heatpump | 7 days | Same as energy |
| homelab.temperature | 7 days | Same as energy |
| homelab.heatpump.settings | 30 days | Audit trail of changes |

## Consumer Groups

### timescaledb-* Groups

Write data to database:

```bash
# View all database sink consumer groups
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group list | grep timescaledb
```

### energy-ws Group

Stream real-time data to WebSockets:

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe energy-ws
```

### Monitoring Lag

High lag indicates consumers are falling behind:

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk group describe timescaledb-energy --output json | \
  jq '.partitions[] | {partition, lag}'
```

**Alert Thresholds:**
- **Warning**: lag > 100 messages
- **Critical**: lag > 1000 messages

## Best Practices

### Topic Naming

- ✅ Use lowercase
- ✅ Use dots for hierarchy
- ✅ Be descriptive
- ❌ Don't use underscores
- ❌ Don't use abbreviations

### Partitioning

- Start with 1-3 partitions
- Partition by key (device_id, sensor_id)
- Don't over-partition (overhead)
- Can increase partitions later (can't decrease)

### Retention

- Balance storage vs replay capability
- Kafka is not primary storage
- Use compression for longer retention
- Consider data volume and disk space

### Compression

- **snappy**: Fast, good compression, recommended
- **gzip**: Better compression, slower
- **lz4**: Fastest, less compression
- **zstd**: Best compression, newer

## Future Topics

Planned for future implementation:

- `homelab.alerts` - System alerts and notifications
- `homelab.weather` - External weather data
- `homelab.automation` - Home automation events
- `homelab.solar` - Solar panel production (if installed)
