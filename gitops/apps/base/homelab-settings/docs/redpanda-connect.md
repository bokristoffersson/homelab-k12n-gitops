# Redpanda Connect Processor

## Overview

The Redpanda Connect instance for homelab-settings acts as a message processor and gateway between the Kafka streaming platform and the device control system.

## Configuration

### Input

Consumes from the homelab settings topic:

```yaml
input:
  kafka:
    addresses: ["redpanda-v2.redpanda-v2.svc.cluster.local:9092"]
    topics: ["homelab.heatpump.settings"]
    consumer_group: homelab-settings
```

### Processing Pipeline

```yaml
pipeline:
  processors:
    # Validate settings
    - mapping: |
        root = this
        root.validated_at = now()

    # Check temperature range
    - bloblang: |
        if this.target_temp < 18 || this.target_temp > 28 {
          throw("Temperature out of range: %v".format(this.target_temp))
        }

    # Add metadata
    - mapping: |
        root.processed_by = "redpanda-connect"
        root.version = "1.0"
```

### Output

Sends validated settings to the heat pump control endpoint:

```yaml
output:
  http_client:
    url: http://heatpump-control:8080/api/settings
    verb: POST
    headers:
      Content-Type: application/json
```

## Monitoring

### Consumer Group Status

Check processing lag:

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk group describe homelab-settings
```

### Logs

View processing logs:

```bash
kubectl logs -n homelab-settings deployment/redpanda-connect -f
```

### Metrics

Available on port 4195:

- `benthos_input_received_total`: Settings requests received
- `benthos_output_sent_total`: Settings applied successfully
- `benthos_processor_error_total`: Validation failures

## Error Handling

### Invalid Settings

If settings fail validation:

1. Error is logged
2. Message is moved to dead-letter queue (if configured)
3. Notification sent to monitoring system

### Connection Failures

If heat pump control is unreachable:

1. Retry with exponential backoff
2. After max retries, move to DLQ
3. Alert sent to operations

## Troubleshooting

### Settings Not Being Applied

1. Check consumer group lag:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe heatpump-settings
   ```

2. Verify device control is accessible:
   ```bash
   kubectl exec -n homelab-settings deployment/redpanda-connect -- \
     curl http://device-control:8080/health
   ```

3. Check for validation errors in logs:
   ```bash
   kubectl logs -n homelab-settings deployment/redpanda-connect | grep ERROR
   ```

### High Processing Latency

1. Check Redpanda Connect resource usage:
   ```bash
   kubectl top pod -n homelab-settings
   ```

2. Scale if needed:
   ```bash
   kubectl scale -n homelab-settings deployment/redpanda-connect --replicas=2
   ```

## Deployment

The service is deployed in the `homelab-settings` namespace:

```bash
kubectl get all -n homelab-settings
```

### Restart

```bash
kubectl rollout restart -n homelab-settings deployment/redpanda-connect
```
