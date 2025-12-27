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
- **JWT Authentication**: Same authentication as redpanda-sink REST API
- **Multi-platform**: Built for linux/amd64 and linux/arm64
- **Low latency**: Direct Kafka → WebSocket path (no database)
- **Scalable**: Supports multiple concurrent clients via broadcast channel

## Deployment

### Prerequisites

1. **Redpanda v2** must be running with `homelab-energy-realtime` topic
2. **Sealed secrets controller** must be installed in the cluster
3. **Docker image** must be available: `ghcr.io/bokristoffersson/energy-ws:main`

### Step 1: Create Sealed Secrets

SSH to your Kubernetes control node and run:

```bash
# Create auth-secret (JWT_SECRET)
kubectl create secret generic auth-secret \
  --namespace=energy-ws \
  --from-literal=JWT_SECRET='0b14cc85ebe2f6177c5540dd806663ff9f9e9d087d80b290aeed98fd4689098a' \
  --dry-run=client -o yaml | \
kubeseal \
  --namespace energy-ws \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/energy-ws/auth-secret-sealed.yaml

# Create ghcr-secret (copy from existing namespace)
kubectl get secret ghcr-secret -n redpanda-sink -o yaml | \
  sed 's/namespace: redpanda-sink/namespace: energy-ws/' | \
  kubectl apply -f -

kubectl get secret ghcr-secret -n energy-ws -o yaml | \
kubeseal \
  --namespace energy-ws \
  --controller-name sealed-secrets \
  --controller-namespace kube-system \
  --format=yaml > gitops/apps/base/energy-ws/ghcr-secret-sealed.yaml
```

### Step 2: Update Kustomization

Uncomment the sealed secret resources in `kustomization.yaml`:

```yaml
resources:
  - namespace.yaml
  - configmap.yaml
  - deployment.yaml
  - service.yaml
  - auth-secret-sealed.yaml
  - ghcr-secret-sealed.yaml
```

### Step 3: Deploy

```bash
# Apply manifests
kubectl apply -k gitops/apps/base/energy-ws/

# OR use Flux to reconcile
flux reconcile kustomization energy-ws --with-source
```

### Step 4: Verify

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
# 1. Missing sealed secrets (auth-secret or ghcr-secret)
# 2. Image pull failures (check ghcr-secret)
# 3. ConfigMap not found
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

### Authentication failures

```bash
# Verify JWT_SECRET matches redpanda-sink
kubectl get secret auth-secret -n energy-ws -o jsonpath='{.data.JWT_SECRET}' | base64 -d
kubectl get secret auth-secret -n redpanda-sink -o jsonpath='{.data.JWT_SECRET}' | base64 -d

# These should be identical for shared authentication
```

## Resource Usage

- **CPU**: 50m request, 500m limit
- **Memory**: 64Mi request, 256Mi limit
- **Storage**: None (stateless service)

## Integration with Frontend

See `/applications/heatpump-web/src/services/websocket.ts` for WebSocket client implementation.

## Related Services

- **redpanda-sink**: REST API for historical energy data (shares JWT authentication)
- **mqtt-input**: Publishes energy data to Redpanda topic
- **heatpump-web**: React frontend that consumes WebSocket stream
