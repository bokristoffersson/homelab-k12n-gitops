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
  jwt_secret: "$(JWT_SECRET)"  # From environment
```

## Local Development

### Prerequisites

- Rust 1.75+
- Docker (for Redpanda)
- kubectl access to cluster (for port-forwarding)

### Running Locally

1. **Port-forward Redpanda:**
   ```bash
   kubectl port-forward -n redpanda-v2 svc/redpanda-v2 9092:9092
   ```

2. **Set JWT secret:**
   ```bash
   export JWT_SECRET=$(kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.JWT_SECRET}' | base64 -d)
   ```

3. **Copy example config:**
   ```bash
   cp config/config.example.yaml config/config.yaml
   # Edit brokers to "localhost:9092" for local development
   ```

4. **Run the service:**
   ```bash
   cargo run
   ```

5. **Test connection:**
   ```bash
   # Get a JWT token from your auth service
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

### End-to-End Tests (requires Docker)
```bash
# Start Redpanda
docker run -d --name redpanda-test -p 9092:9092 \
  docker.redpanda.com/redpandadata/redpanda:v24.2.19 \
  redpanda start --mode dev-container

# Run tests
export REDPANDA_BROKERS=localhost:9092
export JWT_SECRET=test-secret
cargo test --test end_to_end_test -- --ignored

# Cleanup
docker rm -f redpanda-test
```

## WebSocket Protocol

### Connection

```
wss://api.k12n.com/ws/energy?token=<JWT_TOKEN>
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

## Building Docker Image

```bash
docker build -t ghcr.io/bokristoffersson/energy-ws:latest .
docker push ghcr.io/bokristoffersson/energy-ws:latest
```

## Deployment

See `/gitops/apps/base/energy-ws/` for Kubernetes manifests.

```bash
kubectl apply -k gitops/apps/base/energy-ws/
flux reconcile kustomization energy-ws --with-source
```

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
