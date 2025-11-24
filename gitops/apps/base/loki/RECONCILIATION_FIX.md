# Loki Reconciliation Issue - Root Cause and Fix

## Problem
Loki HelmRelease was not reconciling due to a **rollback loop**:

1. **Initial issue**: Upgrade failed with timeout (readiness probe failing)
2. **Remediation triggered**: Because `remediation.retries: 1` was set, Flux tried to rollback
3. **Rollback also failed**: Rollback timed out because pod readiness probe still failing
4. **Stuck in loop**: HelmRelease stuck in "Reconciling" state trying to complete rollback
5. **Kustomization timeout**: Kustomization timed out waiting for HelmRelease to become ready

## Root Cause
- Loki pod readiness probe returning **503 error**
- Helm waits for pods to be ready before completing operations
- Operations timeout after 20 minutes
- Remediation tries rollback, but rollback also fails (same readiness issue)

## Solution Applied

### 1. Removed Remediation Strategy
**Changed**: Removed `install.remediation` and `upgrade.remediation` sections
**Reason**: Remediation causes rollback attempts which fail when readiness probe is broken. Better to fail fast and fix the root cause.

### 2. Kept Increased Timeout
**Kept**: `timeout: 20m`
**Reason**: Gives more time for operations to complete

### 3. Resource Limits Added
**Added**: Resource requests and limits to prevent OOM issues
**Reason**: Resource constraints can cause readiness probe failures

## Current Status

After removing remediation:
- HelmRelease will fail fast instead of getting stuck in rollback loop
- You can manually fix the readiness probe issue
- Once fixed, HelmRelease should reconcile successfully

## Next Steps to Fix Readiness Probe

The readiness probe is still failing (503 error). To fix:

1. **Check Loki configuration**:
   ```bash
   kubectl get configmap loki -n monitoring -o yaml
   ```

2. **Verify MinIO connectivity**:
   ```bash
   kubectl get endpoints loki-minio -n monitoring
   kubectl get pods -n monitoring | grep minio
   ```

3. **Check pod events**:
   ```bash
   kubectl describe pod loki-0 -n monitoring
   kubectl get events -n monitoring --field-selector involvedObject.name=loki-0
   ```

4. **Once readiness probe passes**, HelmRelease should reconcile successfully

## Why Remediation Doesn't Help Here

Remediation (rollback) doesn't help when:
- The issue is with pod readiness (not a configuration problem)
- The same readiness issue affects all versions
- Rollback operations also wait for readiness

Better approach:
- Fail fast on timeout
- Fix the underlying readiness issue
- Then allow upgrade to proceed

