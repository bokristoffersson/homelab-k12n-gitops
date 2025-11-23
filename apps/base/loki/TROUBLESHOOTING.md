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

### Note: Proxy Issues
If you encounter "502 Bad Gateway" or proxy errors when trying to access logs or exec into pods:
- This is a kubelet/proxy connectivity issue between kubectl and the node
- Use alternative methods below to diagnose

1. **Check pod events** (works despite proxy issues):
   ```bash
   kubectl get events -n monitoring --field-selector involvedObject.name=loki-0 --sort-by='.lastTimestamp'
   kubectl describe pod loki-0 -n monitoring
   ```

2. **Check MinIO connectivity**:
   ```bash
   kubectl get endpoints loki-minio -n monitoring
   kubectl get pods -n monitoring | grep minio
   kubectl get svc loki-minio -n monitoring
   ```

3. **Check storage volumes**:
   ```bash
   kubectl get pvc -n monitoring | grep loki
   kubectl describe pvc -n monitoring | grep loki
   ```

4. **Check Loki configuration**:
   ```bash
   kubectl get configmap loki -n monitoring -o yaml | grep -A 30 "storage:"
   ```

5. **Check resource usage**:
   ```bash
   kubectl top pod loki-0 -n monitoring
   ```

6. **Alternative: Access logs via node** (if you have SSH access to the node):
   ```bash
   # SSH to the node running loki-0, then:
   sudo crictl logs <container-id>
   # Or via containerd:
   sudo ctr -n k8s.io containers ls | grep loki
   ```

7. **Check readiness probe status**:
   ```bash
   kubectl get pod loki-0 -n monitoring -o jsonpath='{.status.containerStatuses[?(@.name=="loki")].ready}'
   kubectl get pod loki-0 -n monitoring -o jsonpath='{.status.containerStatuses[?(@.name=="loki")].lastState}'
   ```

## Configuration Changes Made

See `apps/base/loki/helmrelease.yaml`:
- Timeout increased to 20m
- Remediation strategy added
- Resource limits added (requests: 100m CPU, 512Mi memory; limits: 2000m CPU, 4Gi memory)
- HelmRelease unsuspended

### 5. Added Resource Limits
- **Added**: CPU and memory requests/limits to prevent OOM issues
- **Reason**: Unbounded resource usage can cause pods to be killed or hang

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

