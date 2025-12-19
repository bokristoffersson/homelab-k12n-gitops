# MQTT Generator - Usage Guide

## Quick Start (Local Development)

### 1. Build the Docker Image

```bash
cd applications/mqtt-generator
docker build -t mqtt-generator:latest .
```

For k3d local cluster, import the image:

```bash
k3d image import mqtt-generator:latest --cluster homelab-local
```

### 2. Deploy Mosquitto Broker

```bash
# From repository root
kubectl apply -k gitops/infrastructure/controllers-local/mosquitto
```

Wait for Mosquitto to be ready:

```bash
kubectl wait --for=condition=Ready pod -l app=mosquitto -n mosquitto --timeout=60s
```

### 3. Deploy MQTT Generator

```bash
kubectl apply -k gitops/apps/local/mqtt-generator
```

### 4. Subscribe to Messages

In a new terminal, port-forward Mosquitto:

```bash
kubectl port-forward -n mosquitto svc/mosquitto 1883:1883
```

Then subscribe to topics (requires `mosquitto_sub` installed):

```bash
# Subscribe to all topics
mosquitto_sub -h localhost -t 'homelab/#' -v

# Subscribe to specific topic
mosquitto_sub -h localhost -t 'homelab/energy/realtime' -v
```

## Configuration Examples

### Example 1: Simple Energy Monitoring

```json
{
  "streams": [
    {
      "topic": "homelab/energy",
      "interval": 1.0,
      "values": [
        {"name": "power", "min": 0, "max": 5000, "decimals": 2}
      ]
    }
  ]
}
```

Output every second:
```json
{"timestamp": "2025-12-19T10:30:45.123456+00:00", "power": 2543.76}
```

### Example 2: Multi-Sensor Temperature

```json
{
  "streams": [
    {
      "topic": "homelab/sensors/temp",
      "interval": 10.0,
      "values": [
        {"name": "sensor1", "min": 18, "max": 24, "decimals": 1},
        {"name": "sensor2", "min": 18, "max": 24, "decimals": 1},
        {"name": "sensor3", "min": 18, "max": 24, "decimals": 1}
      ]
    }
  ]
}
```

### Example 3: High-Frequency + Low-Frequency

```json
{
  "streams": [
    {
      "topic": "fast/metrics",
      "interval": 0.5,
      "values": [
        {"name": "value", "min": 0, "max": 100, "decimals": 0}
      ]
    },
    {
      "topic": "slow/metrics",
      "interval": 300.0,
      "values": [
        {"name": "daily_avg", "min": 50, "max": 75, "decimals": 2}
      ]
    }
  ]
}
```

## Updating Configuration

Edit the ConfigMap:

```bash
kubectl edit configmap mqtt-generator-config -n mqtt-generator
```

Or update the file and reapply:

```bash
kubectl apply -k gitops/apps/local/mqtt-generator
```

Restart the pod to pick up changes:

```bash
kubectl rollout restart deployment mqtt-generator -n mqtt-generator
```

## Monitoring

### Check Logs

```bash
# Follow logs
kubectl logs -f deployment/mqtt-generator -n mqtt-generator

# Get last 100 lines
kubectl logs --tail=100 deployment/mqtt-generator -n mqtt-generator
```

### Check Status

```bash
# Pod status
kubectl get pods -n mqtt-generator

# Deployment status
kubectl get deployment -n mqtt-generator
```

### Debug

```bash
# Describe pod for events
kubectl describe pod -l app=mqtt-generator -n mqtt-generator

# Get into pod shell
kubectl exec -it deployment/mqtt-generator -n mqtt-generator -- /bin/sh
```

## Testing with MQTT Tools

### Install mosquitto-clients

**macOS:**
```bash
brew install mosquitto
```

**Linux:**
```bash
sudo apt-get install mosquitto-clients
```

### Subscribe Examples

```bash
# All homelab topics with verbose output
mosquitto_sub -h localhost -t 'homelab/#' -v

# JSON formatted (requires jq)
mosquitto_sub -h localhost -t 'homelab/#' | jq .

# Count messages per minute
mosquitto_sub -h localhost -t 'homelab/energy/realtime' | pv -l -r > /dev/null
```

### Publish Test Message

```bash
mosquitto_pub -h localhost -t 'test/message' -m '{"test": "value"}'
```

## Performance Testing

### High-Frequency Test

Generate 1000 messages per second:

```json
{
  "streams": [
    {
      "topic": "perf/test",
      "interval": 0.001,
      "values": [
        {"name": "counter", "min": 0, "max": 1000000, "decimals": 0}
      ]
    }
  ]
}
```

### Monitor Message Rate

```bash
mosquitto_sub -h localhost -t 'perf/test' | pv -l -a > /dev/null
```

## Cleanup

```bash
# Delete generator
kubectl delete -k gitops/apps/local/mqtt-generator

# Delete Mosquitto
kubectl delete -k gitops/infrastructure/controllers-local/mosquitto
```

## Common Issues

### Pod CrashLoopBackOff

Check logs for configuration errors:
```bash
kubectl logs deployment/mqtt-generator -n mqtt-generator
```

### Can't Connect to Mosquitto

Check Mosquitto is running:
```bash
kubectl get pods -n mosquitto
kubectl logs deployment/mosquitto -n mosquitto
```

### Configuration Not Updating

Restart the deployment:
```bash
kubectl rollout restart deployment mqtt-generator -n mqtt-generator
```

### Image Not Found in k3d

Re-import the image:
```bash
k3d image import mqtt-generator:latest --cluster homelab-local
```

## Integration with Other Services

### Send to Redpanda

Configure a consumer to read from MQTT and write to Redpanda topics.

### Store in Time-Series Database

Use a service like Telegraf to consume MQTT and write to InfluxDB/Prometheus.

### Visualize in Grafana

Set up Grafana to query your time-series database.

## Production Considerations

For production use:

1. **Authentication**: Enable MQTT authentication in Mosquitto
2. **TLS**: Use encrypted connections
3. **Resource Limits**: Adjust based on message frequency
4. **Monitoring**: Add Prometheus metrics
5. **Persistence**: Enable Mosquitto persistence for reliability
6. **Rate Limiting**: Configure appropriate intervals to avoid overwhelming consumers
