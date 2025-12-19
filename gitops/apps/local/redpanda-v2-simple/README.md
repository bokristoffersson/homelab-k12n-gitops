# Redpanda Simple - Local Development

Simple Redpanda deployment for local development without operator complexity.

## Installation

```bash
cd gitops/apps/local/redpanda-v2-simple
./install.sh
```

Or use the Makefile:

```bash
make redpanda-install
```

## Create Topics with rpk

### Using Makefile

```bash
# Create all default topics
make rpk-create-topics

# List topics
make rpk-list-topics

# Describe a topic
make rpk-describe-topic TOPIC=homelab-energy
```

### Using rpk directly

```bash
# Get rpk shell access
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- rpk topic list

# Create a topic
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- \
  rpk topic create homelab-energy --partitions 3 --replicas 1

# Describe topic
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- \
  rpk topic describe homelab-energy

# Produce test message
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- \
  rpk topic produce homelab-energy

# Consume messages
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- \
  rpk topic consume homelab-energy --num 10
```

## Default Topics

The following topics are created by default:

- `homelab-energy-realtime` - Energy consumption data (1s interval)
- `homelab-temperature-indoor` - Indoor temperature sensors (30s interval)
- `homelab-temperature-outdoor` - Outdoor weather data (60s interval)
- `homelab-heatpump-status` - Heatpump telemetry (5s interval)

All topics use:
- 3 partitions (good for testing parallelism)
- 1 replica (single node)
- 7 day retention

## Access Console

```bash
# Port-forward console
make port-redpanda

# Open browser
open http://localhost:8080
```

## Check Status

```bash
# Pod status
kubectl get pods -n redpanda-v2

# Redpanda cluster info
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- rpk cluster info

# Health check
kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- rpk cluster health
```

## Uninstall

```bash
# Using Helm
helm uninstall redpanda-v2 -n redpanda-v2

# Or delete namespace (removes everything)
kubectl delete namespace redpanda-v2
```

## Testing MQTT → Redpanda Flow

1. **Deploy MQTT Generator**:
   ```bash
   make mqtt-import
   kubectl apply -k gitops/apps/local/mqtt-generator
   ```

2. **Deploy MQTT to Redpanda Consumer** (if you have one):
   ```bash
   kubectl apply -k gitops/apps/local/mqtt-input-v2
   ```

3. **Monitor messages in Redpanda**:
   ```bash
   # Watch messages arrive
   kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- \
     rpk topic consume homelab-energy-realtime --num 100
   ```

4. **View in Console**:
   - Port-forward: `make port-redpanda`
   - Open http://localhost:8080
   - Navigate to Topics to see messages

## Troubleshooting

### Pod not starting

```bash
kubectl describe pod redpanda-v2-0 -n redpanda-v2
kubectl logs redpanda-v2-0 -n redpanda-v2
```

### Storage issues

```bash
# Check PVC
kubectl get pvc -n redpanda-v2

# k3d uses local-path provisioner by default
```

### Can't connect

```bash
# Port-forward Kafka port
kubectl port-forward redpanda-v2-0 -n redpanda-v2 9092:9092

# Test connection
echo "test" | rpk topic produce test --brokers localhost:9092
```
