# Development Guide

## Prerequisites

- Rust 1.83+ (`rustup install stable`)
- Access to Redpanda cluster
- kubectl access to cluster

## Local Development

### Environment Setup

Create `.env` file:

```bash
KAFKA_BROKERS=localhost:9092
KAFKA_TOPIC=homelab.energy
KAFKA_GROUP_ID=energy-ws-dev
JWKS_URL=https://auth.k12n.com/application/o/energy-ws/jwks/
RUST_LOG=info
WS_PORT=3000
```

### Redpanda Connection

Forward Redpanda port from cluster:

```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-0 9092:9092
```

### Run Development Server

```bash
cargo run
```

Server starts WebSocket endpoint on `ws://localhost:3000/ws`.

### Testing WebSocket

Using wscat:

```bash
# Install wscat
npm install -g wscat

# Connect (token validation skipped in dev mode with proper flag)
wscat -c "ws://localhost:3000/ws?token=dev"
```

## Code Structure

```
src/
├── main.rs              # App initialization, WebSocket handler
├── kafka/
│   ├── consumer.rs      # Redpanda consumer logic
│   └── message.rs       # Message parsing
├── websocket/
│   ├── handler.rs       # WebSocket connection handler
│   └── broadcast.rs     # Message broadcasting
└── auth/
    └── jwt.rs           # JWT validation
```

## Development Workflow

### 1. Start Dependencies

```bash
# Forward Redpanda
kubectl port-forward -n redpanda-v2 svc/redpanda-0 9092:9092

# Optional: Watch Kafka topic
kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic homelab.energy \
  --from-beginning
```

### 2. Run Server

```bash
RUST_LOG=debug cargo run
```

### 3. Connect Client

```bash
wscat -c "ws://localhost:3000/ws?token=dev"
```

### 4. Publish Test Message

```bash
# Publish to Redpanda (requires kafka-console-producer)
echo '{"timestamp":"2026-01-08T19:30:00Z","power_w":2450.5}' | \
  kafka-console-producer \
    --bootstrap-server localhost:9092 \
    --topic homelab.energy
```

## Code Quality

### Format

```bash
cargo fmt
```

### Lint

```bash
cargo clippy --all-targets --all-features
```

### Test

```bash
cargo test --verbose
```

## Adding Features

### New Message Fields

1. Update struct in `src/kafka/message.rs`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyMessage {
    pub timestamp: DateTime<Utc>,
    pub power_w: f64,
    pub new_field: f64,  // Add new field
}
```

2. Parser will handle it automatically (serde magic)

3. Test with updated message format

### Message Filtering

Add filtering logic in `src/kafka/consumer.rs`:

```rust
// Only broadcast if power > threshold
if message.power_w > 100.0 {
    broadcast_tx.send(message).await?;
}
```

### Connection Metrics

Add Prometheus metrics (planned):

```rust
use prometheus::{IntGauge, register_int_gauge};

lazy_static! {
    static ref ACTIVE_CONNECTIONS: IntGauge =
        register_int_gauge!("ws_active_connections", "Active WebSocket connections")
        .unwrap();
}

// On connect
ACTIVE_CONNECTIONS.inc();

// On disconnect
ACTIVE_CONNECTIONS.dec();
```

## Docker Build

Multi-stage Dockerfile with dependency caching:

```bash
docker build --platform linux/arm64 -t energy-ws:test .
```

Build times:
- First build: ~30 minutes (dependencies)
- Subsequent: ~5 minutes (code only)

## Deployment

GitHub Actions workflow automatically:
1. Runs tests on ARM64 runner
2. Builds Docker image
3. Pushes to ghcr.io
4. FluxCD deploys to cluster

## Debugging

### Enable Detailed Logging

```bash
RUST_LOG=energy_ws=debug,rdkafka=info cargo run
```

### Kafka Consumer Issues

Check consumer lag:

```bash
kafka-consumer-groups \
  --bootstrap-server localhost:9092 \
  --describe \
  --group energy-ws-dev
```

### WebSocket Connection Issues

Test with curl:

```bash
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Host: localhost:3000" \
  -H "Origin: http://localhost:3000" \
  http://localhost:3000/ws?token=dev
```

## Performance Tuning

### Kafka Consumer

Adjust fetch settings in `src/kafka/consumer.rs`:

```rust
consumer.set("fetch.min.bytes", "1000");
consumer.set("fetch.wait.max.ms", "100");
```

### WebSocket Buffering

Configure message buffer size:

```rust
let (tx, rx) = tokio::sync::mpsc::channel(100); // Buffer 100 messages
```

### Broadcast Optimization

Use `tokio::select!` for efficient multi-client broadcast:

```rust
tokio::select! {
    Some(msg) = rx.recv() => {
        for client in clients.iter() {
            client.send(msg.clone()).await;
        }
    }
}
```

## Troubleshooting

### "Failed to create consumer"

Check Redpanda is accessible and topic exists:

```bash
kafka-topics --bootstrap-server localhost:9092 --list
```

### "WebSocket connection failed"

1. Verify port 3000 is not in use
2. Check JWT validation isn't blocking
3. Review CORS settings if browser client

### High Memory Usage

1. Limit connected clients
2. Reduce message buffer size
3. Add message rate limiting

## Related Documentation

- [Usage Guide](usage.md) - Client integration examples
- [mqtt-kafka-bridge](../mqtt-kafka-bridge) - Data source
- [Redpanda Docs](https://docs.redpanda.com/) - Kafka compatibility
