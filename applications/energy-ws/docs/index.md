# Energy WebSocket Server

Real-time energy data streaming service via WebSocket protocol.

## Overview

Energy-ws is a Rust/Axum-based WebSocket server that streams real-time power consumption data from Redpanda (Kafka) to connected clients. It enables live energy monitoring dashboards with sub-second latency.

## Key Features

- **Real-time streaming**: Sub-second data delivery via WebSocket
- **Kafka consumer**: Reads from `homelab.energy` topic in Redpanda
- **JWT authentication**: Secure WebSocket connections via Authentik
- **Multiple clients**: Broadcasts to all connected clients
- **Auto-reconnect**: Handles connection drops gracefully
- **Low overhead**: Efficient Rust/Axum implementation

## Technology Stack

- **Language**: Rust 1.83
- **Framework**: Axum with tokio-tungstenite for WebSocket
- **Message Broker**: Redpanda (Kafka-compatible)
- **Consumer Library**: rdkafka
- **Authentication**: JWT validation (Authentik JWKS)
- **Container**: Multi-stage Docker build with ARM64 support

## Architecture

```
┌──────────────────┐
│  Shelly EM       │
│  (Power Meter)   │
└────────┬─────────┘
         │ MQTT
         ▼
┌──────────────────┐
│ mqtt-kafka-bridge│
└────────┬─────────┘
         │ Kafka
         ▼
┌──────────────────┐     WebSocket     ┌──────────────────┐
│ Redpanda         │◄──────────────────│  energy-ws       │
│ homelab.energy   │                   │  (this service)  │
└──────────────────┘                   └────────┬─────────┘
                                                │ WS://
                                                ▼
                                       ┌──────────────────┐
                                       │  Dashboard       │
                                       │  (Browser)       │
                                       └──────────────────┘
```

## Data Flow

1. **Shelly EM** sends power readings to MQTT broker
2. **mqtt-kafka-bridge** publishes to Redpanda topic `homelab.energy`
3. **energy-ws** consumes messages from Redpanda
4. **energy-ws** broadcasts to all connected WebSocket clients
5. **Dashboard** displays real-time power consumption

## WebSocket Protocol

### Connection

Connect to: `wss://energy-ws.k12n.com/ws`

Authentication via query parameter:
```
wss://energy-ws.k12n.com/ws?token=<jwt_token>
```

### Message Format

Server sends JSON messages:

```json
{
  "timestamp": "2026-01-08T19:30:00Z",
  "power_w": 2450.5,
  "voltage": 230.2,
  "current": 10.64,
  "energy_kwh": 145.3
}
```

Messages are sent in real-time as Kafka messages arrive (typically every 1-2 seconds).

### Client Example

```javascript
const token = "your-jwt-token";
const ws = new WebSocket(`wss://energy-ws.k12n.com/ws?token=${token}`);

ws.onopen = () => {
  console.log('Connected to energy stream');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(`Power: ${data.power_w}W`);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected, reconnecting...');
  // Implement reconnect logic
};
```

## Deployment

- **Namespace**: `energy-ws`
- **Replicas**: 1 (stateful Kafka consumer)
- **Resources**: 100m CPU, 128Mi memory
- **Ingress**: Traefik with WebSocket support
- **Consumer Group**: `energy-ws` (single partition, single consumer)

## Performance

- **Latency**: <100ms from Kafka message to WebSocket broadcast
- **Throughput**: Handles 100+ concurrent WebSocket connections
- **Message rate**: ~1-2 messages/second per connection
- **Memory**: ~50MB RSS per pod

## Monitoring

- **Health endpoint**: `/health` for k8s probes
- **Kafka consumer lag**: Tracked via consumer group
- **Active connections**: Logged to stdout
- **Message throughput**: Prometheus metrics (planned)

## Related Components

- **Frontend**: Dashboard consuming WebSocket stream
- **Data source**: [mqtt-kafka-bridge](../mqtt-kafka-bridge) publishes to Redpanda
- **Message broker**: Redpanda cluster
- **Persistence**: [redpanda-sink](../../gitops/apps/base/timescaledb) writes to TimescaleDB
