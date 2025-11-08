# Redpanda Real-Time Data Pipeline Setup

## Overview

This document outlines the setup for real-time heat pump data streaming via Redpanda → Rust consumer → Centrifugo → Mobile app.

## Architecture

```
MQTT (Mosquitto)
  ↓
heatpump-mqtt (Rust) ──┬──→ TimescaleDB (Historical, Persistent)
                       │
                       └──→ Redpanda (Streaming, 1-24h retention)
                            ↓
                       Rust Consumer
                            ↓
                       Centrifugo (WebSocket)
                            ↓
                       Mobile App
```

## Step 1: Create Redpanda Topics

### Real-time Heat Pump Data (1 hour retention)

```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic create heatpump-realtime \
  --partitions 1 \
  --replicas 1 \
  --retention-ms 3600000 \
  --segment-ms 300000

# Configuration:
# - retention-ms: 3600000 (1 hour)
# - segment-ms: 300000 (5 minutes per segment)
# - Partitions: 1 (single heat pump, single device)
```

### Energy Data (24 hour retention)

```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic create energy-realtime \
  --partitions 1 \
  --replicas 1 \
  --retention-ms 86400000 \
  --segment-ms 900000

# Configuration:
# - retention-ms: 86400000 (24 hours)
# - segment-ms: 900000 (15 minutes per segment)
```

### Verify Topics

```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe heatpump-realtime
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe energy-realtime
```

## Step 2: Modify heatpump-mqtt for Dual Write

Your heatpump-mqtt application needs to write to both TimescaleDB (historical) and Redpanda (streaming).

### Add Redpanda Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
rdkafka = { version = "0.36", features = ["cmake-build", "ssl-vendor"] }
# or
rdkafka = { version = "0.36", features = ["cmake-build"] }
```

### Environment Variables

Update `deployment.yaml`:
```yaml
env:
  - name: REDPANDA_ENABLED
    value: "true"
  - name: REDPANDA_BROKERS
    value: "redpanda.redpanda.svc.cluster.local:9092"
  - name: REDPANDA_TOPIC_HEATPUMP
    value: "heatpump-realtime"
  - name: REDPANDA_TOPIC_ENERGY
    value: "energy-realtime"
```

### Implementation Pattern

```rust
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;

struct RedpandaWriter {
    producer: FutureProducer,
    heatpump_topic: String,
    energy_topic: String,
}

impl RedpandaWriter {
    async fn write_heatpump_data(&self, device_id: &str, data: &[u8]) -> Result<()> {
        let record = FutureRecord::to(&self.heatpump_topic)
            .key(device_id.as_bytes())
            .payload(data);
        
        self.producer.send(record, Duration::from_secs(0)).await?;
        Ok(())
    }
    
    async fn write_energy_data(&self, data: &[u8]) -> Result<()> {
        let record = FutureRecord::to(&self.energy_topic)
            .payload(data);
        
        self.producer.send(record, Duration::from_secs(0)).await?;
        Ok(())
    }
}

// In your MQTT processor:
async fn process_message(msg: MqttMessage) -> Result<()> {
    // 1. Write to TimescaleDB (historical)
    timescaledb.insert(&msg).await?;
    
    // 2. Write to Redpanda (streaming)
    if redpanda_enabled {
        redpanda.write_heatpump_data(&msg.device_id, &msg.payload).await?;
    }
    
    Ok(())
}
```

## Step 3: Create Rust Consumer Service

Create a new service that reads from Redpanda and broadcasts to Centrifugo/WebSocket.

### New Application: heatpump-redpanda-consumer

**File Structure:**
```
apps/base/heatpump-redpanda-consumer/
├── deployment.yaml
├── kustomization.yaml
└── configmap.yaml
```

### Consumer Implementation (Simplified)

```rust
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;
use tungstenite::{accept, Message};

async fn consume_and_broadcast() -> Result<()> {
    // Create Kafka consumer
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "redpanda.redpanda.svc.cluster.local:9092")
        .set("group.id", "heatpump-consumer")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "true")
        .create()?;
    
    consumer.subscribe(&["heatpump-realtime", "energy-realtime"])?;
    
    let mut websocket_clients = Vec::new();
    
    loop {
        match consumer.recv().await {
            Ok(msg) => {
                let payload = msg.payload().unwrap();
                // Broadcast to all WebSocket clients
                broadcast_to_websockets(&mut websocket_clients, payload).await?;
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
            }
        }
    }
}
```

## Step 4: Deploy Centrifugo (Optional)

Alternatively, use the rust consumer with built-in WebSocket server.

### Centrifugo HelmRelease

```yaml
apiVersion: helm.toolkit.fluxcd.io/v2
kind: HelmRelease
metadata:
  name: centrifugo
  namespace: centrifugo
spec:
  interval: 10m
  chart:
    spec:
      chart: centrifugo
      sourceRef:
        kind: HelmRepository
        name: centrifugo
        namespace: flux-system
  values:
    enabled: true
    configmap:
      server:
        broker: redpanda
        kafka:
          addrs: ["redpanda.redpanda.svc.cluster.local:9092"]
```

## Step 5: Testing

### Test Produce to Redpanda

```bash
# Produce test heat pump data
kubectl exec -it redpanda-0 -n redpanda -- rpk topic produce heatpump-realtime \
  --key "device-001" \
  <<<'{"device_id":"device-001","timestamp":1698765432,"d0":5.2,"d5":35.0,"d6":30.5}'

# Consume and verify
kubectl exec -it redpanda-0 -n redpanda -- rpk topic consume heatpump-realtime \
  --format '%k: %v\n' \
  --num 10
```

### Test Consumer

```bash
# Check if consumer is connected
kubectl logs -n heatpump-mqtt -l app=heatpump-redpanda-consumer -f

# Check consumer group status
kubectl exec -it redpanda-0 -n redpanda -- rpk group describe heatpump-consumer
```

## Step 6: Monitoring

### Prometheus Metrics

Expose metrics from your consumer:
- `redpanda_messages_consumed_total`
- `redpanda_consumer_lag`
- `redpanda_broadcast_count`
- `redpanda_websocket_clients_connected`

### Key Metrics to Monitor

```promql
# Consumer lag (should be close to 0)
redpanda_consumer_lag

# Messages processed per second
rate(redpanda_messages_consumed_total[5m])

# Active WebSocket connections
redpanda_websocket_clients_connected
```

## Data Flow Examples

### Heat Pump Data Flow

```
MQTT Message → heatpump-mqtt → TimescaleDB (historical)
                              → Redpanda (heatpump-realtime)
                                                   ↓
                                          Rust Consumer
                                                   ↓
                                                   ↓
                                     ┌─────────────┴─────────────┐
                                     ↓                           ↓
                                Mobile App 1              Mobile App 2
```

### Energy Data Flow

```
MQTT Message → heatpump-mqtt → TimescaleDB (historical)
                              → Redpanda (energy-realtime)
                                                   ↓
                                          Rust Consumer
                                                   ↓
                                                   ↓
                                      ┌────────────┴────────────┐
                                      ↓                        ↓
                                 Mobile App 1          Mobile App 2
```

## Retention Strategy

### Redpanda
- **heatpump-realtime**: 1 hour retention
- **energy-realtime**: 24 hour retention
- **Segments**: Automatic cleanup based on retention

### TimescaleDB
- **Raw data**: Keep 1 year
- **Compressed data**: Keep indefinitely
- **Continuous aggregates**: Keep forever

## Mobile App Integration

### WebSocket Connection

```javascript
// React Native / Web
const ws = new WebSocket('wss://your-domain.com/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  // Handle real-time heat pump data
  if (data.topic === 'heatpump-realtime') {
    updateHeatPumpDashboard(data);
  }
  
  // Handle energy data
  if (data.topic === 'energy-realtime') {
    updateEnergyChart(data);
  }
};
```

### API Endpoints (Optional)

For historical data queries:
```javascript
// Get last hour from Redpanda (real-time)
GET /api/v1/heatpump/realtime/last-hour

// Get historical data from TimescaleDB
GET /api/v1/heatpump/history?start=2024-01-01T00:00:00Z&end=2024-01-02T00:00:00Z
```

## Troubleshooting

### Consumer Not Receiving Messages

```bash
# Check consumer group status
kubectl exec -it redpanda-0 -n redpanda -- rpk group describe heatpump-consumer

# Check consumer logs
kubectl logs -n heatpump-mqtt -l app=heatpump-redpanda-consumer
```

### High Consumer Lag

```bash
# Check lag metrics
curl http://consumer-service:8080/metrics | grep redpanda_consumer_lag

# Investigate slow processing
kubectl top pod -n heatpump-mqtt -l app=heatpump-redpanda-consumer
```

### WebSocket Disconnections

```bash
# Check WebSocket server logs
kubectl logs -n heatpump-mqtt -l app=heatpump-redpanda-consumer | grep websocket

# Check connection count
curl http://consumer-service:8080/metrics | grep redpanda_websocket_clients_connected
```

## References

- [Redpanda Documentation](https://docs.redpanda.com/)
- [rdkafka Rust Client](https://docs.rs/rdkafka/)
- [Centrifugo Documentation](https://centrifugal.dev/docs)
- [WebSocket Protocol](https://tools.ietf.org/html/rfc6455)

