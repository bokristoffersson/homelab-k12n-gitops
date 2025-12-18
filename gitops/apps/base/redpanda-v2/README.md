# Redpanda v2

## Accessing the Console

The Redpanda Console is deployed as part of the Redpanda Helm chart without authentication.

To access it locally via port-forwarding:

```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-v2-console 8080:8080
```

Then open your browser to: http://localhost:8080

## Kafka Connection

Applications can connect to Redpanda using:

- **Kafka broker**: `redpanda-v2.redpanda-v2.svc.cluster.local:9092`
- **Admin API**: `http://redpanda-v2.redpanda-v2.svc.cluster.local:9644`
- **HTTP Proxy**: `http://redpanda-v2.redpanda-v2.svc.cluster.local:8082`

## Storage

- **PVC Size**: 50Gi
- **Storage Class**: longhorn
