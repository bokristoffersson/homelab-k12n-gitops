# Redpanda Topics - GitOps Setup

## Overview

This setup manages Redpanda topics declaratively using GitOps (FluxCD). Topics are defined as YAML files in Git and automatically applied to the cluster.

## Structure

```
apps/base/redpanda/
├── namespace.yaml          # Redpanda namespace
├── helmrelease.yaml        # Redpanda Helm release
├── kustomization.yaml      # References topics/ directory
└── topics/
    ├── kustomization.yaml  # Kustomization for topics
    ├── job.yaml            # Job that creates topics
    └── README.md           # Documentation
```

## How It Works

1. **FluxCD watches Git**: Flux monitors the repository for changes
2. **Changes detected**: When you commit new files or modify existing ones
3. **Kustomization applied**: Flux applies the kustomization which includes the topics job
4. **Job executed**: Kubernetes runs a Job that creates the topics
5. **Idempotent**: The Job checks if topics exist before creating them

## Topics Created

### heatpump-realtime
- **Purpose**: Real-time heat pump sensor data
- **Retention**: 1 hour (3,600,000 ms)
- **Partitions**: 1
- **Replicas**: 1
- **Segment**: 5 minutes

### energy-realtime  
- **Purpose**: Real-time energy consumption data
- **Retention**: 24 hours (86,400,000 ms)
- **Partitions**: 1
- **Replicas**: 1
- **Segment**: 15 minutes

## Applying Changes

### 1. Add Files to Git

```bash
# Add the new topics configuration
git add apps/base/redpanda/topics/
git add apps/base/redpanda/kustomization.yaml

# Commit and push
git commit -m "Add Redpanda topics GitOps configuration"
git push
```

### 2. FluxCD Will Automatically Apply

FluxCD will:
1. Detect the new files
2. Apply the kustomization
3. Run the job to create topics
4. Clean up completed jobs after 5 minutes

### 3. Verify Topics Were Created

```bash
# List topics
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list

# Describe a topic
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe heatpump-realtime
```

## Monitoring the Setup

### Check Flux Status

```bash
# Check if kustomization is synced
kubectl get kustomizations -n flux-system

# Check specific kustomization
kubectl describe kustomization -n flux-system redpanda
```

### Check Job Status

```bash
# List jobs
kubectl get jobs -n redpanda

# Check job logs
kubectl logs -n redpanda job/redpanda-topics-setup

# Check job status
kubectl describe job -n redpanda redpanda-topics-setup
```

### Manual Trigger (if needed)

```bash
# Force Flux to reconcile
flux reconcile kustomization redpanda -n flux-system --with-source

# Or trigger the reconciliation via kubectl
kubectl annotate kustomization redpanda -n flux-system \
  reconciler.fluxcd.io/requestedAt="$(date +%s)"
```

## Modifying Topics

### Add a New Topic

1. Edit `apps/base/redpanda/topics/job.yaml`
2. Add a new topic creation block:

```yaml
# Example: Add my-new-topic
if ! rpk topic describe my-new-topic --brokers redpanda.redpanda.svc.cluster.local:9092 2>/dev/null; then
  echo "Creating my-new-topic..."
  rpk topic create my-new-topic \
    --brokers redpanda.redpanda.svc.cluster.local:9092 \
    --partitions 3 \
    --replicas 1 \
    --retention-ms 7200000
fi
```

3. Commit and push
4. Flux will apply the changes

### Modify Topic Settings

**Note**: Modifying job.yaml will NOT update existing topics. You need to:
1. Delete the topic manually
2. Remove the `|| true` check in job.yaml
3. Commit changes to force recreation

Or manually update:
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic alter-config heatpump-realtime \
  --set retention.ms=7200000
```

## Troubleshooting

### Topics Not Created

**Check if Redpanda is ready**:
```bash
kubectl get pods -n redpanda
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=redpanda -n redpanda --timeout=5m
```

**Check job logs**:
```bash
kubectl logs -n redpanda job/redpanda-topics-setup
```

**Manually create topics**:
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk topic create heatpump-realtime \
  --partitions 1 --replicas 1 --retention-ms 3600000
```

### Job Keeps Running

**Check if Redpanda is accessible**:
```bash
kubectl exec -it redpanda-0 -n redpanda -- rpk cluster info
```

**Force job completion**:
```bash
kubectl delete job -n redpanda redpanda-topics-setup
# Flux will recreate it on next reconcile
```

### Flux Not Applying Changes

**Check kustomization status**:
```bash
kubectl get kustomizations -n flux-system -o wide
kubectl describe kustomization -n flux-system redpanda
```

**Force reconciliation**:
```bash
flux reconcile kustomization redpanda -n flux-system
flux reconcile source git flux-system
```

**Check Flux logs**:
```bash
kubectl logs -n flux-system -l app=helm-controller --tail=50
kubectl logs -n flux-system -l app=kustomize-controller --tail=50
```

## Alternative: Manual Topic Creation

If you prefer not to use GitOps for topics, you can manually create them:

```bash
# Create heatpump-realtime
kubectl exec -it redpanda-0 -n redpanda -- rpk topic create heatpump-realtime \
  --partitions 1 --replicas 1 --retention-ms 3600000 --segment-ms 300000

# Create energy-realtime  
kubectl exec -it redpanda-0 -n redpanda -- rpk topic create energy-realtime \
  --partitions 1 --replicas 1 --retention-ms 86400000 --segment-ms 900000

# Verify
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list
```

## Next Steps

1. ✅ Topics are managed via GitOps
2. ⏳ Modify heatpump-mqtt to publish to Redpanda
3. ⏳ Create Rust consumer for real-time data
4. ⏳ Build WebSocket layer (Centrifugo or custom)
5. ⏳ Connect mobile app

See `REDPANDA_SETUP.md` in `apps/base/heatpump-mqtt/` for detailed implementation guide.

