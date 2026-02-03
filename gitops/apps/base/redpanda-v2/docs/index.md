# Redpanda v2

Redpanda is a Kafka-compatible streaming platform deployed in the `redpanda-v2` namespace. It provides high-performance message streaming for the homelab IoT data pipeline.

## Architecture

```
IoT Devices → MQTT (Mosquitto) → Redpanda → Consumers
                                     ↓
                              TimescaleDB (via redpanda-sink)
                                     ↓
                              homelab-api (REST)
```

## Components

### Redpanda Cluster
- **Namespace**: `redpanda-v2`
- **Replicas**: 1 (single-node cluster)
- **Storage**: 50Gi Longhorn PVC
- **Version**: v25.3.1

### Redpanda Console
Web UI for managing and monitoring Redpanda.

**Access via port-forward**:
```bash
kubectl port-forward -n redpanda-v2 svc/redpanda-v2-console 8080:8080
```

Then open: http://localhost:8080

## Connection Endpoints

Applications connect to Redpanda using:

| Protocol | Endpoint | Port |
|----------|----------|------|
| Kafka | `redpanda-v2.redpanda-v2.svc.cluster.local:9092` | 9092 |
| Admin API | `redpanda-v2.redpanda-v2.svc.cluster.local:9644` | 9644 |
| HTTP Proxy | `redpanda-v2.redpanda-v2.svc.cluster.local:8082` | 8082 |

## Topic Management

Topics are managed directly via `rpk` commands in a Kubernetes Job, not through the Redpanda operator or CRDs. This provides a simpler, more reliable architecture.

See [Topics](topics.md) for details on topic configuration and management.

## Monitoring

Redpanda exports Prometheus metrics via ServiceMonitor:
- **Metrics endpoint**: `:9644/metrics`
- **Scrape interval**: 30s

Grafana dashboards are available for monitoring cluster health, throughput, and latency.

## Storage

- **PVC Size**: 50Gi
- **Storage Class**: longhorn
- **Mount Path**: `/var/lib/redpanda/data`

## Resources

- **CPU**: 1 core (limit: 2 cores)
- **Memory**: 2Gi (limit: 4Gi)

## Related Components

- **mqtt-kafka-bridge**: Redpanda Connect pipeline from MQTT to Redpanda
- **redpanda-sink**: Consumer writing telemetry to TimescaleDB
- **homelab-settings-api**: Consumer managing homelab settings
