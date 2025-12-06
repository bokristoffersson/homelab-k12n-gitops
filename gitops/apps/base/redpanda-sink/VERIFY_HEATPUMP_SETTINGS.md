# Verifying Heatpump Settings Static Table

This guide provides step-by-step instructions to verify that the heatpump settings (d50-d58) are being stored correctly in the static table.

## Prerequisites

- `kubectl` configured to access your cluster
- Access to the `redpanda-sink` and `heatpump-mqtt` namespaces
- Database credentials (from `timescaledb-secret`)

## Step 1: Verify Migration Ran Successfully

### Check Migration Job Status

```bash
# Check if the migration job exists and completed
kubectl get jobs -n redpanda-sink redpanda-sink-migration

# View migration job logs
kubectl logs -n redpanda-sink job/redpanda-sink-migration

# Look for successful table creation
kubectl logs -n redpanda-sink job/redpanda-sink-migration | grep -i "heatpump_settings\|completed successfully"
```

### Verify Table Exists in Database

```bash
# Get database credentials
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Check if heatpump_settings table exists
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT table_name, table_type 
FROM information_schema.tables 
WHERE table_schema = 'public' 
  AND table_name = 'heatpump_settings';
"

# View table structure
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "\d heatpump_settings"
```

## Step 2: Verify Pipeline Configuration

### Check ConfigMap is Updated

```bash
# View the configmap to verify heatpump-settings pipeline exists
kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep -A 30 "heatpump-settings"

# Or view the full config
kubectl get configmap redpanda-sink-config -n redpanda-sink -o jsonpath='{.data.config\.yaml}' | grep -A 30 "heatpump-settings"
```

### Verify Deployment is Using Updated Config

```bash
# Check if deployment has restarted (should have after configmap update)
kubectl get pods -n redpanda-sink -l app=redpanda-sink

# Check pod logs for configuration loading
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "heatpump-settings\|pipeline"
```

## Step 3: Check Application Status

### Verify Deployment is Running

```bash
# Check deployment status
kubectl get deployment -n redpanda-sink redpanda-sink

# Check pod status
kubectl get pods -n redpanda-sink -l app=redpanda-sink

# Describe pod for detailed status
kubectl describe pod -n redpanda-sink -l app=redpanda-sink
```

### Check Application Logs

```bash
# Follow application logs in real-time
kubectl logs -n redpanda-sink -f deployment/redpanda-sink

# View recent logs
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=100

# Look for heatpump-settings pipeline activity
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "heatpump-settings"

# Check for errors
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "error\|failed"
```

## Step 4: Verify Redpanda Topic

### Check if Topic Exists

```bash
# List all topics
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic list --brokers localhost:9092

# Check heatpump-settings topic specifically
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic describe heatpump-settings --brokers localhost:9092
```

### Consume Messages from Topic (for testing)

```bash
# Consume a few messages to see the message structure
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic consume heatpump-settings --brokers localhost:9092 --num 5
```

## Step 5: Test with Sample Message

### Send Test Message to Redpanda

If you need to test manually, you can send a test message:

```bash
# Send a test message to the heatpump-settings topic
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic produce heatpump-settings --brokers localhost:9092 <<EOF
{
  "Client_Name": "test-heatpump-01",
  "d50": 22.5,
  "d51": 1,
  "d52": 2,
  "d53": 10,
  "d54": 30,
  "d55": 25,
  "d56": 20,
  "d57": 15,
  "d58": 0
}
EOF
```

### Verify Message was Processed

```bash
# Check application logs for processing
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=50 | grep -i "heatpump-settings\|batch flushed"

# Wait a few seconds, then check database
sleep 5
```

## Step 6: Verify Data in Database

### Check if Data Exists

```bash
# Set up credentials
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Check row count
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT COUNT(*) as total_devices 
FROM heatpump_settings;
"

# View all settings
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  device_id,
  latest_update,
  indoor_target_temp,
  mode,
  curve,
  curve_min,
  curve_max,
  curve_plus5,
  curve_0,
  curve_minus5,
  heatstop
FROM heatpump_settings
ORDER BY latest_update DESC;
"
```

### Test Upsert Functionality

```bash
# Send an update message with the same Client_Name
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic produce heatpump-settings --brokers localhost:9092 <<EOF
{
  "Client_Name": "test-heatpump-01",
  "d50": 23.0,
  "d51": 2,
  "d52": 3,
  "d53": 12,
  "d54": 32,
  "d55": 26,
  "d56": 21,
  "d57": 16,
  "d58": 1
}
EOF

# Wait a few seconds
sleep 5

# Verify the record was updated (not duplicated)
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  device_id,
  latest_update,
  indoor_target_temp,
  mode,
  heatstop
FROM heatpump_settings
WHERE device_id = 'test-heatpump-01';
"

# Should show only ONE row with updated values
```

## Step 7: Check Data Freshness

### Verify Recent Updates

```bash
# Check for recent updates (within last 5 minutes)
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  device_id,
  latest_update,
  NOW() - latest_update as age,
  indoor_target_temp,
  mode
FROM heatpump_settings
WHERE latest_update > NOW() - INTERVAL '5 minutes'
ORDER BY latest_update DESC;
"
```

## Quick Verification Script

Save this as `verify-heatpump-settings.sh`:

```bash
#!/bin/bash

echo "=== Checking Migration Status ==="
kubectl get jobs -n redpanda-sink redpanda-sink-migration

echo -e "\n=== Checking Table Exists ==="
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  COUNT(*) as device_count,
  MAX(latest_update) as most_recent_update,
  NOW() - MAX(latest_update) as age
FROM heatpump_settings;
"

echo -e "\n=== Checking Application Logs ==="
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=20 | grep -i "heatpump-settings\|error" || echo "No recent heatpump-settings activity"

echo -e "\n=== Checking Redpanda Topic ==="
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic describe heatpump-settings --brokers localhost:9092 2>/dev/null || echo "Topic may not exist yet"

echo -e "\n=== Recent Settings Data ==="
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  device_id,
  latest_update,
  indoor_target_temp,
  mode,
  curve
FROM heatpump_settings
ORDER BY latest_update DESC
LIMIT 5;
"
```

Make it executable and run:
```bash
chmod +x verify-heatpump-settings.sh
./verify-heatpump-settings.sh
```

## Troubleshooting

### Table Not Created

1. **Check migration job logs:**
   ```bash
   kubectl logs -n redpanda-sink job/redpanda-sink-migration
   ```

2. **Verify migration file is in ConfigMap:**
   ```bash
   kubectl get configmap redpanda-sink-migrations -n redpanda-sink -o yaml | grep "003_add_heatpump_settings"
   ```

3. **Manually trigger migration job recreation:**
   ```bash
   # Delete the job to force recreation
   kubectl delete job redpanda-sink-migration -n redpanda-sink
   # Flux will recreate it automatically
   ```

### No Data in Table

1. **Check if messages are in Redpanda:**
   ```bash
   kubectl exec -it redpanda-0 -n redpanda -- \
     rpk topic consume heatpump-settings --brokers localhost:9092 --num 1
   ```

2. **Verify pipeline configuration:**
   ```bash
   kubectl get configmap redpanda-sink-config -n redpanda-sink -o jsonpath='{.data.config\.yaml}' | grep -A 20 "heatpump-settings"
   ```

3. **Check application logs for errors:**
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "error\|failed\|heatpump-settings"
   ```

4. **Verify JSONPath matches message structure:**
   - Device ID should come from `$.Client_Name`
   - Settings should be at `$.d50`, `$.d51`, etc.

### Upsert Not Working

1. **Check for duplicate rows:**
   ```bash
   kubectl exec timescaledb-0 -n heatpump-mqtt -- \
     env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
   SELECT device_id, COUNT(*) 
   FROM heatpump_settings 
   GROUP BY device_id 
   HAVING COUNT(*) > 1;
   "
   ```

2. **Verify upsert_key configuration:**
   ```bash
   kubectl get configmap redpanda-sink-config -n redpanda-sink -o jsonpath='{.data.config\.yaml}' | grep -A 5 "upsert_key"
   ```

3. **Check application logs for upsert errors:**
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "upsert\|conflict"
   ```

### Configuration Not Applied

1. **Check if ConfigMap was updated:**
   ```bash
   kubectl get configmap redpanda-sink-config -n redpanda-sink -o yaml | grep "heatpump-settings"
   ```

2. **Restart deployment to pick up new config:**
   ```bash
   kubectl rollout restart deployment/redpanda-sink -n redpanda-sink
   ```

3. **Verify deployment restarted:**
   ```bash
   kubectl get pods -n redpanda-sink -l app=redpanda-sink
   ```

## Expected Results

When everything is working correctly, you should see:

1. ✅ Migration job completed successfully
2. ✅ `heatpump_settings` table exists in database
3. ✅ ConfigMap contains `heatpump-settings` pipeline
4. ✅ Application logs show pipeline is active
5. ✅ Redpanda topic `heatpump-settings` exists (if messages are being published)
6. ✅ Data appears in `heatpump_settings` table
7. ✅ Updates to same `device_id` result in upsert (no duplicates)
