# Migration Plan - Old Setup → New Telemetry Stack

This document outlines the plan for migrating from the old telemetry setup to the new stack.

## Current Status ✅

The new stack is **FULLY DEPLOYED and OPERATIONAL**:

- ✅ **mqtt-kafka-bridge**: Running (renamed from mqtt-redpanda-bridge)
  - 4 active streams: energy, temperature-indoor, temperature-outdoor, heatpump
  - Publishing to Kafka topics successfully

- ✅ **timescaledb**: Running
  - 11,117 heatpump telemetry rows and growing
  - Migration system in place
  - Hypertables configured with 90-day retention

- ✅ **heatpump-settings**: Running
  - Settings stored with UPSERT pattern
  - Migration system in place

## Old Setup (To Be Replaced)

Based on your current deployment, the old setup includes:
- `mqtt-input` or `mqtt-input-v2` pods in their respective namespaces
- Old database pods (if any) in `heatpump-mqtt` namespace
- Old Redpanda sink applications

## Migration Strategy

### Option A: Parallel Operation (Recommended)

Run both systems side-by-side for validation before cutover.

**Timeline**: 7-14 days of parallel operation

**Advantages**:
- Zero downtime
- Data validation before cutover
- Easy rollback
- Confidence in new system

**Disadvantages**:
- Higher resource usage temporarily
- More monitoring needed

### Option B: Direct Cutover

Stop old system and rely entirely on new system.

**Timeline**: Immediate

**Advantages**:
- Lower resource usage
- Simpler
- Faster completion

**Disadvantages**:
- Potential data gap if issues arise
- Harder to validate
- More risk

---

## Detailed Migration Steps

### Phase 1: Validation (Current State)

**Duration**: 1-2 hours

#### 1.1 Verify New System Health

```bash
# Check all pods are running
kubectl get pods -n mqtt-kafka-bridge
kubectl get pods -n timescaledb
kubectl get pods -n heatpump-settings

# Verify all streams active
kubectl logs -n mqtt-kafka-bridge -l app=mqtt-kafka-bridge --tail=20 | grep "active"

# Expected output: 8 lines showing input/output active for 4 streams
```

#### 1.2 Verify Data Flow

```bash
# Check Kafka topics have recent messages
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk topic consume homelab-heatpump-telemetry --num=1 --offset=-1 --brokers=localhost:9092

# Check TimescaleDB has recent data
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c \
  "SELECT COUNT(*), MAX(time) FROM heatpump_status;"

# Check heatpump settings
kubectl exec -n heatpump-settings postgres-75dcb54ff7-2gdn8 -- \
  psql -U postgres -d heatpump_settings -c \
  "SELECT device_id, indoor_target_temp, mode, updated_at FROM settings;"
```

**Success Criteria**:
- ✅ All pods 1/1 Running
- ✅ All 4 streams showing as active
- ✅ Kafka topics have messages from last 5 minutes
- ✅ Database has data from last 5 minutes
- ✅ No errors in logs

---

### Phase 2: Identify Old Resources

**Duration**: 30 minutes

#### 2.1 List Old Deployments

```bash
# Find old MQTT input applications
kubectl get deployments --all-namespaces | grep -E "mqtt-input"

# Find old database pods
kubectl get pods -n heatpump-mqtt | grep timescaledb

# Find old sink applications
kubectl get deployments --all-namespaces | grep -E "redpanda-sink|sink"
```

#### 2.2 Document Old Resources

Create a list of resources to be removed:

**Example** (update based on your findings):
```
OLD RESOURCES TO REMOVE:
- namespace: mqtt-input
  - deployment: mqtt-input
- namespace: mqtt-input-v2
  - deployment: mqtt-input-v2
- namespace: redpanda-sink
  - deployment: redpanda-connect
- namespace: redpanda-sink-v2
  - deployment: redpanda-connect
- namespace: heatpump-mqtt (CAREFUL - has old timescaledb)
  - statefulset: timescaledb-0
```

---

### Phase 3: Parallel Operation (If Option A)

**Duration**: 7-14 days

#### 3.1 Monitor Both Systems

```bash
# Create monitoring script
cat > monitor-both-systems.sh <<'EOF'
#!/bin/bash
echo "=== NEW SYSTEM ==="
echo "mqtt-kafka-bridge:"
kubectl get pods -n mqtt-kafka-bridge
echo ""
echo "TimescaleDB rows:"
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c "SELECT COUNT(*) FROM heatpump_status;" | grep -v "count\|rows\|-"
echo ""

echo "=== OLD SYSTEM ==="
echo "mqtt-input (if exists):"
kubectl get pods -n mqtt-input 2>/dev/null || echo "Not found"
echo ""
echo "Old TimescaleDB (if exists):"
kubectl get pods -n heatpump-mqtt | grep timescaledb || echo "Not found"
EOF

chmod +x monitor-both-systems.sh

# Run daily
./monitor-both-systems.sh
```

#### 3.2 Compare Data Quality

```bash
# Check data consistency
# Compare row counts, timestamps, values between old and new systems

# New system
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c \
  "SELECT
    COUNT(*) as total_rows,
    MIN(time) as earliest,
    MAX(time) as latest,
    COUNT(DISTINCT device_id) as devices
   FROM heatpump_status;"

# Old system (adjust namespace/pod name)
# kubectl exec -n heatpump-mqtt timescaledb-0 -- \
#   psql -U postgres -c "SELECT COUNT(*), MIN(time), MAX(time) FROM ..."
```

#### 3.3 Validation Checklist

Daily for 7-14 days:

- [ ] New system pods all healthy
- [ ] Data arriving continuously (no gaps)
- [ ] Data matches old system (if comparable)
- [ ] No errors in logs
- [ ] Resource usage stable
- [ ] Disk space growing at expected rate

---

### Phase 4: Prepare for Cutover

**Duration**: 1-2 hours

#### 4.1 Final Health Check

```bash
# Run comprehensive health check
echo "=== FINAL HEALTH CHECK ==="

# 1. Pod health
kubectl get pods -n mqtt-kafka-bridge -o wide
kubectl get pods -n timescaledb -o wide
kubectl get pods -n heatpump-settings -o wide

# 2. Recent data
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c \
  "SELECT
    'heatpump' as table_name,
    COUNT(*) as rows,
    MAX(time) as latest_data,
    NOW() - MAX(time) as data_age
   FROM heatpump_status;"

# 3. Log check (last 100 lines should have no errors)
kubectl logs -n mqtt-kafka-bridge -l app=mqtt-kafka-bridge --tail=100 | grep -i error
kubectl logs -n timescaledb -l app=redpanda-connect --tail=100 | grep -i error
kubectl logs -n heatpump-settings -l app=redpanda-connect --tail=100 | grep -i error

# Expected: No errors found
```

#### 4.2 Backup Old Data (If Applicable)

```bash
# If you have old TimescaleDB with historical data you want to keep
kubectl exec -n heatpump-mqtt timescaledb-0 -- \
  pg_dump -U postgres > old-timescaledb-backup-$(date +%Y%m%d).sql

# Compress
gzip old-timescaledb-backup-*.sql

# Store somewhere safe (S3, NAS, etc.)
```

#### 4.3 Document Current State

```bash
# Capture current state before changes
kubectl get all --all-namespaces | grep -E "mqtt-input|mqtt-kafka|timescaledb|heatpump-settings|redpanda-sink" > pre-cutover-state.txt

# Resource usage
kubectl top pods --all-namespaces >> pre-cutover-state.txt
```

---

### Phase 5: Cutover

**Duration**: 30 minutes - 1 hour

⚠️ **IMPORTANT**: Perform during low-traffic period if possible

#### 5.1 Scale Down Old Deployments

```bash
# Scale down old MQTT inputs (DOES NOT delete data)
kubectl scale deployment mqtt-input --replicas=0 -n mqtt-input 2>/dev/null
kubectl scale deployment mqtt-input-v2 --replicas=0 -n mqtt-input-v2 2>/dev/null

# Scale down old sinks
kubectl scale deployment redpanda-connect --replicas=0 -n redpanda-sink 2>/dev/null
kubectl scale deployment redpanda-connect --replicas=0 -n redpanda-sink-v2 2>/dev/null

echo "Old applications scaled to 0 replicas"
```

#### 5.2 Monitor New System

```bash
# Watch logs for 15 minutes
kubectl logs -n mqtt-kafka-bridge -l app=mqtt-kafka-bridge -f &
kubectl logs -n timescaledb -l app=redpanda-connect -f &

# Let run for 15 minutes, verify:
# - No errors
# - Data still flowing
# - All streams active
```

#### 5.3 Verify No Data Loss

```bash
# Check data is still arriving
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c \
  "SELECT
    COUNT(*) FILTER (WHERE time > NOW() - INTERVAL '5 minutes') as last_5min,
    COUNT(*) FILTER (WHERE time > NOW() - INTERVAL '15 minutes') as last_15min,
    MAX(time) as latest
   FROM heatpump_status;"

# Should show recent data
```

**If Any Issues**: Immediately scale old deployments back up:
```bash
kubectl scale deployment mqtt-input --replicas=1 -n mqtt-input
# etc.
```

---

### Phase 6: Update GitOps (Remove Old Apps from Flux)

**Duration**: 30 minutes

#### 6.1 Remove Old App Definitions

```bash
# Edit gitops/clusters/homelab/apps.yaml
# Remove or comment out old kustomizations:

# - name: mqtt-input
# - name: mqtt-input-v2
# - name: redpanda-sink
# - name: redpanda-sink-v2
```

**Example edit**:
```yaml
# OLD - Comment out or remove:
# ---
# apiVersion: kustomize.toolkit.fluxcd.io/v1
# kind: Kustomization
# metadata:
#   name: mqtt-input
#   namespace: flux-system
# spec:
#   interval: 10m0s
#   sourceRef:
#     kind: GitRepository
#     name: flux-system
#   path: ./gitops/apps/homelab/mqtt-input
#   prune: true
#   wait: true
```

#### 6.2 Commit and Push

```bash
git add gitops/clusters/homelab/apps.yaml
git commit -m "chore: remove old mqtt-input and redpanda-sink from Flux

The new mqtt-kafka-bridge, timescaledb, and heatpump-settings stack
is now the primary telemetry pipeline.

Old applications scaled to 0 and removed from Flux management."

git push origin main
```

#### 6.3 Reconcile Flux

```bash
flux reconcile source git flux-system
flux reconcile kustomization flux-system

# Verify old kustomizations are gone
kubectl get kustomizations -n flux-system | grep -E "mqtt-input|redpanda-sink"
# Should return nothing
```

---

### Phase 7: Observation Period

**Duration**: 7 days

#### 7.1 Monitor Daily

```bash
# Daily health check
echo "=== Daily Health Check $(date) ==="

# 1. Pod status
kubectl get pods -n mqtt-kafka-bridge -n timescaledb -n heatpump-settings

# 2. Data growth
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c \
  "SELECT
    pg_size_pretty(pg_total_relation_size('heatpump_status')) as table_size,
    COUNT(*) as rows,
    MAX(time) as latest
   FROM heatpump_status;"

# 3. Resource usage
kubectl top pods -n mqtt-kafka-bridge
kubectl top pods -n timescaledb
kubectl top pods -n heatpump-settings

# 4. PVC usage
kubectl get pvc -n timescaledb -n heatpump-settings
```

#### 7.2 Observation Checklist

- [ ] Day 1: No errors, data flowing
- [ ] Day 3: Resource usage stable
- [ ] Day 5: Data retention working (check old data exists)
- [ ] Day 7: All good, ready for cleanup

---

### Phase 8: Cleanup Old Resources

⚠️ **ONLY after 7+ days of stable operation**

**Duration**: 1-2 hours

#### 8.1 Final Backup of Old Data

```bash
# If not already done, backup old TimescaleDB
kubectl exec -n heatpump-mqtt timescaledb-0 -- \
  pg_dump -U postgres > final-old-timescaledb-backup-$(date +%Y%m%d).sql

gzip final-old-timescaledb-backup-*.sql

# Verify backup
gunzip -c final-old-timescaledb-backup-*.sql.gz | head -50

# Store safely
```

#### 8.2 Delete Old Deployments

```bash
# Delete old MQTT inputs
kubectl delete namespace mqtt-input 2>/dev/null
kubectl delete namespace mqtt-input-v2 2>/dev/null

# Delete old sinks
kubectl delete namespace redpanda-sink 2>/dev/null
kubectl delete namespace redpanda-sink-v2 2>/dev/null

# CAREFUL with heatpump-mqtt - it has your old TimescaleDB!
# Only delete after confirming backup and new system is good
# kubectl delete namespace heatpump-mqtt
```

#### 8.3 Remove Old GitOps Directories

```bash
# Remove old app directories
rm -rf gitops/apps/base/mqtt-input
rm -rf gitops/apps/base/mqtt-input-v2
rm -rf gitops/apps/base/redpanda-sink
rm -rf gitops/apps/base/redpanda-sink-v2

rm -rf gitops/apps/homelab/mqtt-input
rm -rf gitops/apps/homelab/mqtt-input-v2
rm -rf gitops/apps/homelab/redpanda-sink
rm -rf gitops/apps/homelab/redpanda-sink-v2

# Commit
git add -A
git commit -m "chore: remove old telemetry stack GitOps configs"
git push origin main
```

---

## Rollback Procedure

If you need to rollback **before Phase 8 cleanup**:

### Immediate Rollback

```bash
# 1. Scale old deployments back up
kubectl scale deployment mqtt-input --replicas=1 -n mqtt-input
kubectl scale deployment redpanda-connect --replicas=1 -n redpanda-sink

# 2. Scale new deployments down (optional)
kubectl scale deployment mqtt-kafka-bridge --replicas=0 -n mqtt-kafka-bridge
kubectl scale deployment redpanda-connect --replicas=0 -n timescaledb
kubectl scale deployment redpanda-connect --replicas=0 -n heatpump-settings

# 3. Monitor old system
kubectl logs -n mqtt-input -f
```

### Rollback After Cleanup

If you deleted old resources but need to recover:

```bash
# 1. Restore from backup
kubectl create namespace mqtt-input
kubectl apply -f <old-deployment-configs>

# 2. If you backed up old database
kubectl create namespace heatpump-mqtt
# Deploy old TimescaleDB
# Restore from pg_dump backup

# 3. Re-enable in Flux
# Uncomment old apps in gitops/clusters/homelab/apps.yaml
git add gitops/clusters/homelab/apps.yaml
git commit -m "rollback: restore old telemetry stack"
git push origin main
```

---

## Success Criteria

Migration is complete when:

- ✅ New system running for 7+ days with no issues
- ✅ Data flowing continuously (verified by checking latest timestamps)
- ✅ No errors in logs
- ✅ Resource usage stable and acceptable
- ✅ Old data backed up (if needed)
- ✅ Old deployments scaled to 0 or deleted
- ✅ Old GitOps configs removed
- ✅ Team comfortable with new system

---

## Post-Migration Tasks

### 1. Update Documentation

- [ ] Update architecture diagrams
- [ ] Update runbooks
- [ ] Document new database schemas
- [ ] Update monitoring dashboards

### 2. Configure Backups

```bash
# Set up automated backups for new TimescaleDB
# See DEPLOYMENT_PLAN.md "Backup Strategy" section
```

### 3. Set Up Alerts

If using Prometheus/Alertmanager:

```yaml
# Example alert rules
- alert: TimescaleDBNoRecentData
  expr: |
    (time() - max(timestamp(heatpump_status))) > 600
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "No heatpump data in last 10 minutes"

- alert: MQTTKafkaBridgeDown
  expr: |
    up{job="mqtt-kafka-bridge"} == 0
  for: 5m
  labels:
    severity: critical
```

### 4. Optimize Retention Policies

```sql
-- After a month, review data growth and adjust retention
-- Connect to TimescaleDB
SELECT
  hypertable_name,
  pg_size_pretty(hypertable_size(format('%I.%I', hypertable_schema, hypertable_name))) as size
FROM timescaledb_information.hypertables;

-- Adjust retention if needed (currently 90 days)
SELECT remove_retention_policy('heatpump_status');
SELECT add_retention_policy('heatpump_status', INTERVAL '60 days');
```

---

## Timeline Summary

| Phase | Duration | Description |
|-------|----------|-------------|
| Phase 1 | 1-2 hours | Validation of new system |
| Phase 2 | 30 min | Identify old resources |
| Phase 3 | 7-14 days | Parallel operation (Option A) |
| Phase 4 | 1-2 hours | Prepare for cutover |
| Phase 5 | 30-60 min | Cutover (scale down old) |
| Phase 6 | 30 min | Update GitOps |
| Phase 7 | 7 days | Observation period |
| Phase 8 | 1-2 hours | Cleanup |

**Total**: ~2-3 weeks for safe migration with Option A (parallel operation)

**Total**: ~1 day for Option B (direct cutover) - higher risk

---

## Troubleshooting

### Issue: New system shows gaps in data after cutover

**Diagnosis**:
```bash
# Check for gaps in last hour
kubectl exec -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry -c \
  "SELECT
    time_bucket('1 minute', time) as bucket,
    COUNT(*) as records
   FROM heatpump_status
   WHERE time > NOW() - INTERVAL '1 hour'
   GROUP BY bucket
   ORDER BY bucket DESC;"
```

**Solution**:
- Check mqtt-kafka-bridge logs for MQTT connection issues
- Verify Redpanda is accessible
- Check network policies aren't blocking traffic

### Issue: Old data not preserved

**Recovery**:
```bash
# Restore from backup
gunzip -c old-timescaledb-backup-*.sql.gz | \
  kubectl exec -i -n timescaledb timescaledb-58cc96cb4f-rzdln -- \
  psql -U postgres -d telemetry
```

### Issue: High resource usage after migration

**Actions**:
```bash
# Check resource usage
kubectl top pods --all-namespaces | grep -E "mqtt|timescale|heatpump"

# Scale down if needed
kubectl scale deployment mqtt-kafka-bridge --replicas=1 -n mqtt-kafka-bridge

# Adjust resource limits in deployment-patch.yaml
# Then apply changes
```

---

## Contact

Questions or issues? Check:
1. This migration plan
2. [DEPLOYMENT_PLAN.md](./DEPLOYMENT_PLAN.md) for detailed deployment info
3. GitHub issues: https://github.com/bokristoffersson/homelab-k12n-gitops/issues

---

**Last Updated**: 2025-12-26
**Status**: New system deployed, ready for migration planning
