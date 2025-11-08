# Redpanda Topics - GitOps Managed

This directory contains the GitOps configuration for managing Redpanda topics using `rpk` CLI via Kubernetes Jobs.

## Why rpk CLI Instead of CRDs?

The Redpanda Helm chart **doesn't include an operator or Topic CRDs**. Using `rpk` CLI is the standard approach for managing topics in a Helm-based deployment. This is:
- ✅ GitOps friendly (FluxCD managed)
- ✅ Idempotent (safe to run multiple times)  
- ✅ No additional operators needed
- ✅ Matches best practices for Helm deployments

See `OPTIONS.md` for a comparison of different approaches.

## How It Works

1. **Job Creation**: A Kubernetes Job is created that sets up the Redpanda topics
2. **Flux Reconciliation**: FluxCD monitors the Git repository and applies changes
3. **Idempotent**: The job checks if topics exist before creating them
4. **Automatic Cleanup**: Jobs are automatically cleaned up after completion (optional)

## Topics Managed

- `heatpump-realtime`: Real-time heat pump data
  - Retention: 1 hour
  - Partitions: 1
  - Replicas: 1

- `energy-realtime`: Real-time energy consumption data  
  - Retention: 24 hours
  - Partitions: 1
  - Replicas: 1

## Triggering Topic Creation

### Automatic (via Flux)
When you commit and push changes to this directory, FluxCD will automatically:
1. Apply the Job
2. The Job will create topics if they don't exist
3. Jobs are cleaned up after 5 minutes by default

### Manual Reconcile
To force reconcile the topics:

```bash
flux reconcile kustomization redpanda -n flux-system
```

Or via Flux CLI:
```bash
flux reconcile source git flux-system
```

## Adding New Topics

1. Edit `job.yaml` to add a new topic creation command
2. Commit and push to Git
3. FluxCD will automatically apply the changes

Example:
```bash
if ! rpk topic describe my-new-topic --brokers redpanda.redpanda.svc.cluster.local:9092 2>/dev/null; then
  echo "Creating my-new-topic..."
  rpk topic create my-new-topic \
    --brokers redpanda.redpanda.svc.cluster.local:9092 \
    --partitions 1 \
    --replicas 1 \
    --retention-ms 3600000
fi
```

## Troubleshooting

### Jobs Keep Running
If jobs are stuck, check logs:
```bash
kubectl logs -n redpanda job/redpanda-topics-setup
```

### Topics Not Created
1. Check if Redpanda is ready:
   ```bash
   kubectl get pods -n redpanda
   ```

2. Manually trigger the job:
   ```bash
   kubectl delete job -n redpanda redpanda-topics-setup
   # Flux will recreate it automatically
   ```

3. Or run the commands manually:
   ```bash
   kubectl exec -it redpanda-0 -n redpanda -- rpk topic list
   ```

### Force Reconcile
To force Flux to reconcile:
```bash
flux reconcile kustomization redpanda -n flux-system --with-source
```

