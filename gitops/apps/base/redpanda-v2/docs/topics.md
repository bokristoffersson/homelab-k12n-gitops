# Topics

Topics in Redpanda are managed directly via `rpk` commands executed in a Kubernetes Job. This approach eliminates the need for the Redpanda operator and provides a simpler, more reliable architecture.

## Management Approach

**Traditional (removed)**:
- Redpanda operator + Topic CRDs
- Complex operator reconciliation
- Kubernetes-native but heavyweight

**Current (rpk-based)**:
- Kubernetes Job with `rpk topic create` commands
- Direct Redpanda API interaction
- Simpler, more reliable, idempotent

## Topic Configuration

### Production Topics

| Topic | Partitions | Replicas | Retention | Purpose |
|-------|------------|----------|-----------|---------|
| `energy-realtime` | 1 | 1 | 24 hours | Real-time energy consumption data |
| `heatpump-realtime` | 1 | 1 | 1 hour | Real-time heatpump telemetry |
| `homelab-settings` | 1 | 1 | 7 days | Homelab settings configuration changes |
| `heatpump-telemetry` | 1 | 1 | 1 hour | General heatpump telemetry |
| `sensor-state` | 1 | 1 | 24 hours | IoT sensor state changes |

### Legacy Topics

These topics were created before the naming convention was standardized:

- `homelab-energy-realtime`
- `homelab-heatpump-telemetry`
- `homelab-temperature-indoor`
- `homelab-temperature-outdoor`

## Topic Creator Job

Topics are created via a Kubernetes Job that runs on startup:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: create-topics
  namespace: redpanda-v2
spec:
  ttlSecondsAfterFinished: 300  # Auto-cleanup after 5 minutes
  template:
    spec:
      containers:
        - name: rpk
          image: docker.redpanda.com/redpandadata/redpanda:v25.3.1
          command: [/bin/sh, -c]
          args:
            - |
              # Wait for Redpanda to be ready
              until rpk cluster info --brokers redpanda-v2.redpanda-v2.svc.cluster.local:9092; do
                sleep 5
              done

              # Create topics (idempotent - skips if exists)
              rpk topic create <topic-name> \\
                --brokers redpanda-v2.redpanda-v2.svc.cluster.local:9092 \\
                --partitions 1 \\
                --replicas 1 \\
                --topic-config retention.ms=<retention-ms>
```

**Location**: `gitops/apps/base/redpanda-v2/topics/topic-creator-job.yaml`

## Adding New Topics

To add a new topic:

1. Edit `gitops/apps/base/redpanda-v2/topics/topic-creator-job.yaml`
2. Add a new `rpk topic create` command with:
   - Topic name
   - Partitions (usually 1 for single-broker)
   - Replicas (usually 1 for single-broker)
   - Retention period (`retention.ms`)
   - Segment size (`segment.ms`)

3. Commit and push changes
4. FluxCD will apply the updated Job
5. Delete the old Job to trigger recreation:
   ```bash
   kubectl delete job create-topics -n redpanda-v2
   ```

## Managing Topics Manually

### List Topics

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -c redpanda -- \\
  rpk topic list
```

### Describe Topic

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -c redpanda -- \\
  rpk topic describe <topic-name>
```

### Update Topic Configuration

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -c redpanda -- \\
  rpk topic alter-config <topic-name> \\
    --set retention.ms=86400000
```

### Delete Topic

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -c redpanda -- \\
  rpk topic delete <topic-name>
```

## Consumer Groups

| Consumer Group | Topics | Purpose |
|----------------|--------|---------|
| `homelab-settings` | `homelab-heatpump-telemetry` | Redpanda-connect processor |
| `homelab-settings-api` | `homelab-heatpump-telemetry` | Rust-based processor |
| `timescaledb-sink` | `homelab-heatpump-telemetry`, `homelab-energy-realtime` | Writes to TimescaleDB |

## Monitoring

View topic metrics in:
- **Redpanda Console**: http://localhost:8080 (via port-forward)
- **Prometheus**: Metrics exported on `:9644/metrics`
- **Grafana**: Custom dashboards for topic throughput and lag

## Troubleshooting

### Job Failed

Check Job logs:
```bash
kubectl logs -n redpanda-v2 job/create-topics
```

Common issues:
- Redpanda not ready (Job will retry)
- Network connectivity to broker
- Invalid topic configuration

### Topic Not Created

Verify Redpanda is running:
```bash
kubectl get pods -n redpanda-v2
kubectl exec -n redpanda-v2 redpanda-v2-0 -c redpanda -- rpk cluster info
```

### Recreate Topics

Delete the Job and recreate with timestamp:
```bash
kubectl delete job create-topics -n redpanda-v2

sed "s/TIMESTAMP_PLACEHOLDER/$(date +%s)/g" \\
  gitops/apps/base/redpanda-v2/topics/topic-creator-job.yaml | kubectl apply -f -
```
