# Data Flow

## End-to-End Data Pipeline

This document explains how data flows from IoT devices to visualization dashboards.

## 1. Device Data Collection

### MQTT Publishing

IoT devices publish data to the Mosquitto MQTT broker:

```
Device → MQTT PUBLISH → mosquitto.mosquitto.svc:1883
```

**Example Energy Meter Data:**
```json
{
  "topic": "homelab/energy/meter1",
  "payload": {
    "timestamp": "2026-01-03T16:00:00Z",
    "total_energy_kwh": 12.45,
    "power_kw": 3.2,
    "voltage": 230,
    "current": 13.9
  }
}
```

## 2. MQTT to Kafka Bridge

The MQTT-Kafka bridge subscribes to MQTT topics and publishes to Kafka:

```
Mosquitto → MQTT Subscribe → MQTT-Kafka Bridge → Kafka Publish → Redpanda
```

**Topic Mapping:**
- `homelab/energy/#` → `homelab.energy`
- `homelab/heatpump/#` → `homelab.heatpump`
- `homelab/temperature/#` → `homelab.temperature`

**Enrichment:**
- Add processing timestamp
- Add source metadata
- Validate payload structure

## 3. Kafka Distribution

Redpanda distributes messages to multiple consumers:

```
Redpanda Topic
    ├── Consumer Group: timescaledb-energy
    ├── Consumer Group: energy-ws
    └── Consumer Group: analytics (future)
```

**Benefits:**
- Multiple consumers process same data independently
- Each consumer tracks its own offset
- No data duplication in storage

## 4. Data Storage (TimescaleDB)

Redpanda Connect sinks consume from Kafka and write to TimescaleDB:

```
Kafka → Redpanda Sink → TimescaleDB Table
```

**Processing Steps:**
1. Consume message from Kafka
2. Parse and validate JSON
3. Transform to table schema
4. INSERT into hypertable
5. Commit Kafka offset

**Example SQL Insert:**
```sql
INSERT INTO energy_readings (timestamp, device_id, total_energy_kwh, power_kw)
VALUES ('2026-01-03T16:00:00Z', 'meter1', 12.45, 3.2);
```

## 5. Real-Time Streaming (WebSocket)

The Energy WebSocket server consumes from Kafka and streams to browsers:

```
Kafka → energy-ws Consumer → WebSocket → Browser
```

**Features:**
- Real-time updates (< 1s latency)
- Multiple concurrent clients
- Message filtering by device
- Automatic reconnection

## 6. API Access

Applications query historical data via Homelab API:

```
Browser → Homelab API → TimescaleDB → JSON Response
```

**Example API Request:**
```
GET /api/v1/energy/latest
GET /api/v1/energy/hourly?start=2026-01-03T00:00:00Z&end=2026-01-03T23:59:59Z
```

## 7. Visualization

### Grafana

Grafana queries TimescaleDB directly for dashboards:

```
Grafana → PostgreSQL Query → TimescaleDB → Visualization
```

**Query Example:**
```sql
SELECT
  time_bucket('1 hour', timestamp) AS hour,
  AVG(power_kw) AS avg_power
FROM energy_readings
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY hour
ORDER BY hour;
```

### Web Dashboards

React applications fetch data from Homelab API:

```
Heatpump Web SPA → HTTP GET → Homelab API → JSON → Chart Rendering
```

## Data Latency

| Stage | Latency | Description |
|-------|---------|-------------|
| Device → MQTT | < 100ms | IoT device publishes |
| MQTT → Kafka | < 200ms | Bridge processing |
| Kafka → TimescaleDB | < 500ms | Sink batch write |
| Kafka → WebSocket | < 100ms | Real-time stream |
| **Total (end-to-end)** | **< 1s** | Device to visualization |

## Data Volume

Approximate message rates:

- **Energy Meters**: 1 msg/sec per meter (2 meters) = 2 msg/sec
- **Heat Pump**: 1 msg/30sec = 0.03 msg/sec
- **Temperature Sensors**: 1 msg/5min per sensor (5 sensors) = 0.017 msg/sec

**Total**: ~2-3 messages/second, ~200K messages/day

## Failure Handling

### Message Loss Prevention

1. **Kafka Replication**: 3 replicas per partition
2. **Consumer Offsets**: Committed only after processing
3. **Dead Letter Queues**: Failed messages retained
4. **Database Transactions**: ACID compliance

### Retry Logic

```
Kafka Message → Process → Success?
                         ├─ Yes → Commit Offset
                         └─ No → Retry (3x) → DLQ
```

## Monitoring Data Flow

### Check End-to-End

1. **Device Publishing**:
   ```bash
   mosquitto_sub -h mosquitto.mosquitto.svc -t "homelab/#"
   ```

2. **Kafka Topics**:
   ```bash
   rpk topic consume homelab.energy --num 10
   ```

3. **Consumer Lag**:
   ```bash
   rpk group describe timescaledb-energy
   ```

4. **Database Writes**:
   ```sql
   SELECT COUNT(*) FROM energy_readings WHERE timestamp > NOW() - INTERVAL '1 minute';
   ```

5. **WebSocket Streaming**:
   - Open browser DevTools → Network → WS
   - Watch messages flowing in real-time
