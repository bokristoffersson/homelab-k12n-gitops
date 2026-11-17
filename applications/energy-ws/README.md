# Energy WebSocket Service

Real-time energy data streaming service that consumes from Redpanda and broadcasts to WebSocket clients.

## Architecture

```
Redpanda (homelab-energy-realtime topic)
         ↓
  Kafka Consumer (energy-ws)
         ↓
  Broadcast Channel (tokio)
         ↓
  WebSocket Clients (authenticated with JWT)
```

## Features

- **Real-time streaming**: ~1 message per second from MQTT → Kafka → WebSocket
- **JWT authentication**: Secure WebSocket connections using same token as REST API
- **Multiple clients**: Broadcast to unlimited concurrent clients
- **Automatic reconnection**: Clients can reconnect without data loss
- **Protocol**: Simple JSON-based subscribe/data/ping/pong protocol

## Configuration

Configuration is loaded from `config/config.yaml` with environment variable substitution:

```yaml
kafka:
  brokers: "redpanda-v2.redpanda-v2.svc.cluster.local:9092"
  topic: "homelab-energy-realtime"
  group_id: "energy-ws"
  auto_offset_reset: "latest"

server:
  host: "0.0.0.0"
  port: 8080
  max_connections: 1000

auth:
  issuers:
    - name: authelia
      issuer: "https://auth.k12n.com"
      jwks_url: "https://auth.k12n.com/jwks.json"
```

## Local Development

### Prerequisites

- Rust (current stable)
- kubectl access to cluster (for port-forwarding)

### Running Locally

1. **Port-forward Redpanda:**
   ```bash
   kubectl port-forward -n redpanda-v2 svc/redpanda-v2 9092:9092
   ```

2. **Copy example config:**
   ```bash
   cp config/config.example.yaml config/config.yaml
   # Edit brokers to "localhost:9092" for local development
   ```

3. **Run the service:**
   ```bash
   cargo run
   ```

4. **Test connection:**
   ```bash
   # An Authelia access token issued as a JWT (clients oauth2-proxy or homelab-macos)
   TOKEN="your-jwt-token"

   # Connect with wscat
   wscat -c "ws://localhost:8080/ws/energy?token=$TOKEN"
   ```

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test integration_test
```

## WebSocket Protocol

### Connection

```
wss://homelab.k12n.com/ws/energy?token=<JWT_TOKEN>
```

### Client → Server Messages

**Subscribe to stream:**
```json
{"type": "subscribe", "streams": ["energy"]}
```

**Ping:**
```json
{"type": "ping"}
```

### Server → Client Messages

**Real-time data:**
```json
{
  "type": "data",
  "stream": "energy",
  "timestamp": "2025-12-26T12:34:56.789Z",
  "data": {
    "ts": "2025-12-26T14:46:27.88864083Z",
    "fields": {
      "active_power_total": 1289,
      "active_power_l1": 1048,
      "active_power_l2": 190,
      "active_power_l3": 51,
      "voltage_l1": 236,
      "voltage_l2": 237,
      "voltage_l3": 239
    }
  }
}
```

**Pong response:**
```json
{
  "type": "pong",
  "timestamp": "2025-12-26T12:34:56.789Z"
}
```

**Error:**
```json
{
  "type": "error",
  "message": "Invalid token",
  "code": "UNAUTHORIZED"
}
```

## Build and deploy

`.github/workflows/energy-ws.yml` tests every PR and, on `main`, builds a multi-arch
image and pushes it to GHCR as `:main`. Pick up a new image with:

```bash
kubectl rollout restart deployment/energy-ws -n energy-ws
```

The manifests are in `gitops/apps/energy-ws/` and are applied by Flux.

## Monitoring

**Check consumer lag:**
```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe energy-ws
```

**View logs:**
```bash
kubectl logs -n energy-ws -l app=energy-ws -f
```

**Check connected clients:**
```bash
kubectl logs -n energy-ws -l app=energy-ws | grep "WebSocket client connected"
```

## License

MIT
