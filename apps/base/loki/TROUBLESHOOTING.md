# Loki Installation Troubleshooting

## Issue Summary
Flux was hanging during Loki installation/upgrade due to:
1. HelmRelease upgrade timing out after 10 minutes
2. Loki pod readiness probe failing (503 error) for 5+ days
3. No remediation strategy configured

## Fixes Applied

### 1. Increased Timeout
- **Changed**: `timeout: 10m` → `timeout: 20m`
- **Reason**: Helm upgrades were taking longer than 10 minutes and timing out

### 2. Added Remediation Strategy
- Added `install.remediation` with 1 retry and 5m timeout
- Added `upgrade.remediation` with 1 retry and 5m timeout
- **Reason**: Allows automatic retry on transient failures

### 3. Temporarily Suspended and Resumed
- Suspended HelmRelease to stop Flux from hanging
- Fixed configuration issues
- Resumed HelmRelease

### 4. Pod Restart
- Deleted the stuck `loki-0` pod to force a fresh restart

## Current Status

**HelmRelease**: Reconciling (not stalled anymore)
**Pod Status**: Running but readiness probe still failing

## Remaining Issues

### Readiness Probe Failing (503 Error)
The Loki pod is running but not passing its readiness probe. The `/ready` endpoint is returning 503.

**Possible Causes:**
1. Loki cannot connect to MinIO for storage
2. Loki configuration issue
3. Resource constraints
4. Storage volume issues

**Next Steps to Diagnose:**

1. **Check Loki logs** (when accessible):
   ```bash
   kubectl logs loki-0 -n monitoring -c loki
   ```

2. **Check MinIO connectivity**:
   ```bash
   kubectl get endpoints loki-minio -n monitoring
   kubectl get pods -n monitoring | grep minio
   ```

3. **Check storage volumes**:
   ```bash
   kubectl get pvc -n monitoring | grep loki
   ```

4. **Test Loki readiness endpoint**:
   ```bash
   kubectl exec loki-0 -n monitoring -c loki -- wget -qO- http://localhost:3100/ready
   ```

5. **Check resource usage**:
   ```bash
   kubectl top pod loki-0 -n monitoring
   ```

## Configuration Changes Made

See `apps/base/loki/helmrelease.yaml`:
- Timeout increased to 20m
- Remediation strategy added
- HelmRelease unsuspended

## Monitoring Progress

```bash
# Check HelmRelease status
kubectl get helmrelease loki -n monitoring

# Check pod status
kubectl get pods -n monitoring | grep loki

# Check Flux Kustomization
kubectl get kustomization loki -n flux-system

# Watch for events
kubectl get events -n monitoring --sort-by='.lastTimestamp' | tail -20
```

## If Flux Still Hangs

1. **Temporarily suspend the HelmRelease**:
   ```bash
   kubectl patch helmrelease loki -n monitoring --type merge -p '{"spec":{"suspend":true}}'
   ```

2. **Fix the underlying issue** (readiness probe failure)

3. **Resume the HelmRelease**:
   ```bash
   kubectl patch helmrelease loki -n monitoring --type merge -p '{"spec":{"suspend":false}}'
   ```

