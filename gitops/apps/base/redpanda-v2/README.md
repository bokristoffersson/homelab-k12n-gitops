# Redpanda v2

Kafka-compatible streaming platform for homelab IoT data pipeline.

## Quick Access

**Redpanda Console** (via port-forward):
```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-v2-console 8080:8080
```
Open: http://localhost:8080

## Connection Endpoints

- **Kafka**: `redpanda-v2.redpanda-v2.svc.cluster.local:9092`
- **Admin API**: `redpanda-v2.redpanda-v2.svc.cluster.local:9644`
- **HTTP Proxy**: `redpanda-v2.redpanda-v2.svc.cluster.local:8082`

## Topics

Topics are managed via `rpk` commands in a Kubernetes Job (not operator/CRDs).

**List topics**:
```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -c redpanda -- rpk topic list
```

See [TechDocs](./docs/topics.md) for full topic management guide.

## Architecture

```
IoT → MQTT → Redpanda → TimescaleDB
                ↓
           homelab-api
```

## Documentation

- [Overview](./docs/index.md)
- [Topics](./docs/topics.md)

## Storage

- **PVC Size**: 50Gi
- **Storage Class**: longhorn
- **Location**: `/var/lib/redpanda/data`
