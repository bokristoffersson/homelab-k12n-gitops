# Energy WebSocket Service (energy-ws)

Real-time WebSocket service for streaming energy data from Redpanda to web clients.

## Architecture

```
Redpanda (homelab-energy-realtime topic)
    ↓
Kafka Consumer (energy-ws)
    ↓
Tokio Broadcast Channel
    ↓
WebSocket Clients (authenticated with JWT)
```

## Features

- **Real-time streaming**: WebSocket connections receive energy data as it arrives in Kafka
- **JWT Authentication**: Authelia access tokens validated via JWKS (RS256)
- **Multi-platform**: Built for linux/amd64 and linux/arm64
- **Low latency**: Direct Kafka → WebSocket path (no database)
- **Scalable**: Supports multiple concurrent clients via broadcast channel

## Deployment

### Prerequisites

1. **Redpanda v2** must be running with `homelab-energy-realtime` topic
2. **Docker image** must be available: `ghcr.io/bokristoffersson/energy-ws:main` (public, no pull secret)

No secrets are needed: tokens are validated against Authelia's JWKS endpoint.

### Deploy

```bash
flux reconcile kustomization energy-ws --with-source
```

### Verify

```bash
# Check pod status
kubectl get pods -n energy-ws

# Check logs
kubectl logs -n energy-ws -l app=energy-ws -f

# Check Kafka consumer group lag
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe energy-ws
```

## WebSocket Protocol

### Connection

Connect to: `ws://energy-ws.energy-ws.svc.cluster.local:8080/ws/energy?token=<JWT_TOKEN>`

For external access via Cloudflare Tunnel: `wss://api.k12n.com/ws/energy?token=<JWT_TOKEN>`

### Client → Server Messages

```json
// Subscribe to energy stream
{"type": "subscribe", "streams": ["energy"]}

// Unsubscribe
{"type": "unsubscribe", "streams": ["energy"]}

// Ping (keepalive)
{"type": "ping"}
```

### Server → Client Messages

```json
// Real-time energy data
{
  "type": "data",
  "stream": "energy",
  "timestamp": "2025-12-27T12:34:56.789Z",
  "data": {
    "ts": "2025-12-27T12:34:56.789Z",
    "fields": {
      "consumption_total_w": 2450.5,
      "consumption_L1_actual_w": 816.1,
      "consumption_L2_actual_w": 815.8,
      "consumption_L3_actual_w": 816.3
    }
  }
}

// Pong response
{"type": "pong", "timestamp": "2025-12-27T12:34:56.789Z"}

// Subscription confirmed
{"type": "subscribed", "stream": "energy"}

// Unsubscription confirmed
{"type": "unsubscribed", "stream": "energy"}

// Error
{"type": "error", "message": "Invalid token", "code": "UNAUTHORIZED"}
```

## Configuration

Configuration is managed via ConfigMap (`configmap.yaml`):

- **Kafka brokers**: `redpanda-v2.redpanda-v2.svc.cluster.local:9092`
- **Kafka topic**: `homelab-energy-realtime`
- **Consumer group**: `energy-ws`
- **Offset reset**: `latest` (only new messages for real-time streaming)
- **Max connections**: 1000 concurrent WebSocket clients

## Monitoring

```bash
# View logs
kubectl logs -n energy-ws -l app=energy-ws -f

# Check consumer lag (should be near 0 for real-time)
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe energy-ws

# Check pod resource usage
kubectl top pod -n energy-ws

# Test WebSocket connection
wscat -c "ws://energy-ws.energy-ws.svc.cluster.local:8080/ws/energy?token=$TOKEN"
```

## Troubleshooting

### Pod not starting

```bash
# Check pod events
kubectl describe pod -n energy-ws -l app=energy-ws

# Common issues:
# 1. Image pull failures
# 2. ConfigMap not found
# 3. Authelia JWKS endpoint unreachable at startup
```

### No data received on WebSocket

```bash
# Check if Kafka consumer is connected
kubectl logs -n energy-ws -l app=energy-ws | grep "Subscribed to Kafka"

# Check consumer lag
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe energy-ws

# Verify topic has data
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic consume homelab-energy-realtime --num 1
```

## Resource Usage

- **CPU**: 50m request, 500m limit
- **Memory**: 64Mi request, 256Mi limit
- **Storage**: None (stateless service)

## Integration with Frontend

See `/applications/heatpump-web/src/services/websocket.ts` for WebSocket client implementation.

## Related Services

- **homelab-api**: REST API for historical energy data (same Authelia issuer)
- **mqtt-input**: Publishes energy data to Redpanda topic
- **heatpump-web**: React frontend that consumes WebSocket stream
