# Production Deployment Plan - Telemetry Stack

This document outlines the plan for deploying the new telemetry stack to production alongside the existing setup.

## Overview

The new stack includes:
- **mqtt-redpanda-bridge**: MQTT to Redpanda bridge (replaces mqtt-input)
- **timescaledb**: Time-series telemetry storage
- **heatpump-settings**: Settings state storage
- **Database migration system**: GitOps-controlled schema changes

## Parallel Deployment Strategy

Run the new stack in parallel with the existing setup to:
- Validate data integrity before cutover
- Compare metrics between old and new systems
- Enable easy rollback if issues arise
- Zero downtime migration

## Prerequisites

### 1. Secrets Management

Create the following secrets in the production cluster:

#### TimescaleDB Secrets
```bash
# Create namespace first
kubectl create namespace timescaledb

# Create database credentials secret
kubectl create secret generic timescaledb-credentials \
  --namespace=timescaledb \
  --from-literal=POSTGRES_USER=postgres \
  --from-literal=POSTGRES_PASSWORD='<GENERATE_STRONG_PASSWORD>' \
  --from-literal=POSTGRES_DB=postgres
```

#### Heatpump Settings Secrets
```bash
# Create namespace
kubectl create namespace heatpump-settings

# Create database credentials secret
kubectl create secret generic postgres-credentials \
  --namespace=heatpump-settings \
  --from-literal=POSTGRES_USER=postgres \
  --from-literal=POSTGRES_PASSWORD='<GENERATE_STRONG_PASSWORD>' \
  --from-literal=POSTGRES_DB=postgres
```

#### MQTT Credentials (if not already exists)
```bash
# Create namespace
kubectl create namespace mqtt-redpanda-bridge

# Only if you have MQTT authentication enabled
kubectl create secret generic mqtt-credentials \
  --namespace=mqtt-redpanda-bridge \
  --from-literal=MQTT_USER='' \
  --from-literal=MQTT_PASSWORD=''
```

### 2. Storage Configuration

#### Production Storage Class
Update storage configuration in production overlay:

```yaml
# gitops/apps/homelab/timescaledb/persistentvolumeclaim-patch.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: timescaledb-data
  namespace: timescaledb
spec:
  storageClassName: <YOUR_PRODUCTION_STORAGE_CLASS>  # e.g., longhorn, ceph-block, etc.
  resources:
    requests:
      storage: 50Gi  # Adjust based on expected data volume
```

```yaml
# gitops/apps/homelab/heatpump-settings/persistentvolumeclaim-patch.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-data
  namespace: heatpump-settings
spec:
  storageClassName: <YOUR_PRODUCTION_STORAGE_CLASS>
  resources:
    requests:
      storage: 10Gi
```

### 3. Resource Limits

Update resource limits for production workloads:

```yaml
# gitops/apps/homelab/timescaledb/deployment-patch.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: timescaledb
  namespace: timescaledb
spec:
  template:
    spec:
      containers:
        - name: timescaledb
          resources:
            requests:
              cpu: 500m
              memory: 1Gi
            limits:
              cpu: 2000m
              memory: 4Gi
```

## Deployment Steps

### Phase 1: Infrastructure Setup

1. **Create Production Overlay**
   ```bash
   # Create homelab overlay directory structure
   mkdir -p gitops/apps/homelab/mqtt-redpanda-bridge
   mkdir -p gitops/apps/homelab/timescaledb
   mkdir -p gitops/apps/homelab/heatpump-settings
   ```

2. **Configure Production Overlays**

   Create `gitops/apps/homelab/mqtt-redpanda-bridge/kustomization.yaml`:
   ```yaml
   apiVersion: kustomize.config.k8s.io/v1beta1
   kind: Kustomization
   resources:
     - ../../base/mqtt-redpanda-bridge

   patches:
     - path: deployment-patch.yaml
   ```

   Create `gitops/apps/homelab/mqtt-redpanda-bridge/deployment-patch.yaml`:
   ```yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: mqtt-redpanda-bridge
     namespace: mqtt-redpanda-bridge
   spec:
     template:
       spec:
         containers:
           - name: redpanda-connect
             resources:
               requests:
                 cpu: 200m
                 memory: 256Mi
               limits:
                 cpu: 1000m
                 memory: 512Mi
   ```

   Repeat similar structure for timescaledb and heatpump-settings.

3. **Update Production Redpanda Addresses**

   In production configs, update Redpanda addresses from:
   ```yaml
   addresses:
     - redpanda-v2-0.redpanda-v2.redpanda-v2.svc.cluster.local:9093
   ```

   To your production Redpanda address (if different).

### Phase 2: Database Migration Setup

1. **Deploy TimescaleDB**
   ```bash
   kubectl apply -k gitops/apps/homelab/timescaledb
   ```

2. **Verify Migration Job**
   ```bash
   # Wait for migration job to complete
   kubectl wait --for=condition=complete job/timescaledb-migration \
     --namespace=timescaledb \
     --timeout=120s

   # Check migration logs
   kubectl logs -n timescaledb job/timescaledb-migration

   # Verify migrations applied
   kubectl exec -n timescaledb deployment/timescaledb -- \
     psql -U postgres -d telemetry -c "SELECT * FROM schema_migrations;"
   ```

3. **Deploy Heatpump Settings**
   ```bash
   kubectl apply -k gitops/apps/homelab/heatpump-settings

   # Verify migration
   kubectl wait --for=condition=complete job/postgres-migration \
     --namespace=heatpump-settings \
     --timeout=120s
   ```

### Phase 3: Deploy Data Pipeline

1. **Deploy MQTT-Redpanda Bridge**
   ```bash
   kubectl apply -k gitops/apps/homelab/mqtt-redpanda-bridge
   ```

2. **Verify All Streams Active**
   ```bash
   kubectl logs -n mqtt-redpanda-bridge deployment/mqtt-redpanda-bridge --tail=50
   ```

   Expected output should show:
   - "Input type mqtt is now active" for all 4 streams
   - "Output type kafka is now active" for all 4 streams

3. **Verify Data Flow**
   ```bash
   # Check Redpanda topics have data
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
     rpk topic consume homelab-energy-realtime -n 5

   # Check TimescaleDB has data
   kubectl exec -n timescaledb deployment/timescaledb -- \
     psql -U postgres -d telemetry -c \
     "SELECT COUNT(*) FROM energy_consumption;"

   # Check heatpump settings
   kubectl exec -n heatpump-settings deployment/postgres -- \
     psql -U postgres -d heatpump_settings -c \
     "SELECT * FROM settings;"
   ```

### Phase 4: Monitoring & Validation

1. **Monitor Resource Usage**
   ```bash
   # Check pod resource usage
   kubectl top pods -n timescaledb
   kubectl top pods -n heatpump-settings
   kubectl top pods -n mqtt-redpanda-bridge

   # Check PVC usage
   kubectl get pvc -n timescaledb
   kubectl get pvc -n heatpump-settings
   ```

2. **Validate Data Integrity**

   Compare data between old and new systems:
   ```bash
   # Old system (if applicable)
   # <commands to query old system>

   # New system
   kubectl exec -n timescaledb deployment/timescaledb -- \
     psql -U postgres -d telemetry -c \
     "SELECT time, device_serial, active_power_total
      FROM energy_consumption
      ORDER BY time DESC LIMIT 10;"
   ```

3. **Monitor Logs for Errors**
   ```bash
   # Watch all pods for errors
   kubectl logs -n mqtt-redpanda-bridge deployment/mqtt-redpanda-bridge -f
   kubectl logs -n timescaledb deployment/redpanda-connect -f
   kubectl logs -n heatpump-settings deployment/redpanda-connect -f
   ```

### Phase 5: Parallel Operation Period

1. **Run Both Systems** for at least 7 days
   - Monitor data consistency
   - Compare metrics
   - Verify no data loss
   - Test edge cases (restarts, failures, etc.)

2. **Create Dashboards** (if using Grafana/similar)
   - Connect to new TimescaleDB datasource
   - Replicate existing dashboards
   - Compare metrics side-by-side

### Phase 6: Cutover (When Ready)

1. **Stop Old System** (if replacing mqtt-input)
   ```bash
   kubectl scale deployment mqtt-input --replicas=0 -n mqtt-input
   ```

2. **Monitor New System**
   - Ensure no errors
   - Verify all data streams active
   - Check disk space trends

3. **Clean Up Old Resources** (after validation period)
   ```bash
   # Only after confirming new system is stable
   kubectl delete namespace mqtt-input  # or whatever the old namespace was
   ```

## Rollback Plan

If issues arise, rollback is straightforward since systems run in parallel:

1. **Keep Old System Running**
   - Don't scale down or delete old deployments
   - Old data pipeline continues unaffected

2. **Stop New System**
   ```bash
   kubectl scale deployment mqtt-redpanda-bridge --replicas=0 -n mqtt-redpanda-bridge
   kubectl scale deployment redpanda-connect --replicas=0 -n timescaledb
   kubectl scale deployment redpanda-connect --replicas=0 -n heatpump-settings
   ```

3. **Investigate and Fix**
   - Review logs
   - Fix configuration issues
   - Test in local environment
   - Redeploy when ready

## Security Hardening

### For Production Deployment:

1. **Use Secrets Instead of Plain Values**

   Update deployments to reference secrets:
   ```yaml
   env:
     - name: POSTGRES_PASSWORD
       valueFrom:
         secretKeyRef:
           name: timescaledb-credentials
           key: POSTGRES_PASSWORD
   ```

2. **Network Policies**

   Create network policies to restrict access:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: timescaledb-policy
     namespace: timescaledb
   spec:
     podSelector:
       matchLabels:
         app: timescaledb
     policyTypes:
       - Ingress
     ingress:
       - from:
           - namespaceSelector:
               matchLabels:
                 name: timescaledb
           - namespaceSelector:
               matchLabels:
                 name: grafana  # if you have monitoring
   ```

3. **Enable TLS for PostgreSQL** (optional but recommended)

## Backup Strategy

### TimescaleDB Backups

1. **Automated pg_dump**
   ```bash
   # Create CronJob for daily backups
   kubectl create cronjob timescaledb-backup \
     --image=postgres:16-alpine \
     --schedule="0 2 * * *" \
     --namespace=timescaledb \
     -- /bin/sh -c "pg_dump -h timescaledb -U postgres telemetry > /backups/telemetry-\$(date +%Y%m%d).sql"
   ```

2. **Volume Snapshots**
   - Use your storage provider's snapshot feature
   - Schedule daily snapshots of PVCs
   - Retain for 30 days

### Heatpump Settings Backups

```bash
# Since it's small, daily dumps are sufficient
kubectl exec -n heatpump-settings deployment/postgres -- \
  pg_dump -U postgres heatpump_settings > heatpump_settings_backup.sql
```

## Success Criteria

Before considering deployment complete:

- ✅ All migration jobs complete successfully
- ✅ All pods running and healthy
- ✅ Data flowing to all three tables (energy, temperature, heatpump)
- ✅ No errors in logs for 24 hours
- ✅ Resource usage within expected limits
- ✅ Data matches between old and new systems (if parallel)
- ✅ Backups configured and tested
- ✅ Monitoring dashboards updated

## Support & Troubleshooting

### Common Issues

**Issue: Migration job fails**
```bash
# Check migration job logs
kubectl logs -n timescaledb job/timescaledb-migration

# Delete and recreate if needed
kubectl delete job timescaledb-migration -n timescaledb
kubectl apply -k gitops/apps/homelab/timescaledb
```

**Issue: No data appearing in database**
```bash
# Check Redpanda Connect logs
kubectl logs -n timescaledb deployment/redpanda-connect

# Verify Redpanda topics exist
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic list

# Check MQTT bridge is publishing
kubectl logs -n mqtt-redpanda-bridge deployment/mqtt-redpanda-bridge
```

**Issue: High memory usage**
```bash
# Adjust resource limits
# Edit deployment-patch.yaml and increase limits

# Or scale down temporarily
kubectl scale deployment timescaledb --replicas=0 -n timescaledb
```

## Timeline Estimate

- **Phase 1 (Setup)**: 2-4 hours
- **Phase 2 (Database Migration)**: 1-2 hours
- **Phase 3 (Data Pipeline)**: 2-3 hours
- **Phase 4 (Monitoring)**: 1 hour initial, then ongoing
- **Phase 5 (Parallel Operation)**: 7-14 days
- **Phase 6 (Cutover)**: 1 hour

**Total**: ~1 week of parallel operation + ~1 day of active work

## Contact & Escalation

If you encounter issues during deployment:
1. Check logs (commands above)
2. Review this document's troubleshooting section
3. Check GitHub issues: https://github.com/bokristoffersson/homelab-k12n-gitops/issues
4. Rollback if critical (see Rollback Plan above)
