# Verifying Data Flow in redpanda-sink

This guide provides commands to verify that data is being written to the database by redpanda-sink.

## Prerequisites

- `kubectl` configured to access your cluster
- Access to the `redpanda-sink` and `heatpump-mqtt` namespaces
- Database credentials (from `timescaledb-secret`)

## Step 1: Check Application Status

### Verify Deployment is Running

```bash
# Check if the deployment is running
kubectl get deployment -n redpanda-sink redpanda-sink

# Check pod status
kubectl get pods -n redpanda-sink -l app=redpanda-sink

# Describe the pod for detailed status
kubectl describe pod -n redpanda-sink -l app=redpanda-sink
```

### Verify Migration Job Completed

```bash
# Check migration job status
kubectl get jobs -n redpanda-sink redpanda-sink-migration

# Check migration job logs
kubectl logs -n redpanda-sink job/redpanda-sink-migration

# Verify tables were created
kubectl logs -n redpanda-sink job/redpanda-sink-migration | grep "completed successfully"
```

## Step 2: Check Application Logs

### View Real-time Logs

```bash
# Follow application logs
kubectl logs -n redpanda-sink -f deployment/redpanda-sink

# View recent logs (last 100 lines)
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=100
```

### Look for Key Indicators

```bash
# Check for successful connections
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "connected"

# Check for message processing
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "batch flushed"

# Check for errors
kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "error\|failed"
```

## Step 3: Verify Data in Database

### Get Database Credentials

```bash
# Get database user
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)

# Get database password
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)

# Get database name
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

echo "User: $DB_USER"
echo "Database: $DB_NAME"
```

### Connect to Database

```bash
# Connect via pod exec (recommended)
# Note: Use -h localhost to force TCP connection and set PGPASSWORD
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME"
```

### Check Table Existence

```bash
# Using kubectl exec with proper authentication
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT table_name 
FROM information_schema.tables 
WHERE table_schema = 'public' 
  AND table_name LIKE '%_new';
"
```

### Verify Data is Being Written

```bash
# Set up credentials
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Check heatpump_new table
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT COUNT(*) as total_rows, 
       MIN(ts) as earliest, 
       MAX(ts) as latest 
FROM heatpump_new;
"

# View recent heatpump data
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT ts, device_id, outdoor_temp, supplyline_temp, compressor_on 
FROM heatpump_new 
ORDER BY ts DESC 
LIMIT 10;
"

# Check telemetry_new table
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT COUNT(*) as total_rows,
       MIN(ts) as earliest,
       MAX(ts) as latest
FROM telemetry_new;
"

# View recent sensor data
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT ts, sensor, location, temperature_c, humidity_pct 
FROM telemetry_new 
ORDER BY ts DESC 
LIMIT 10;
"

# Check energy_new table
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT COUNT(*) as total_rows,
       MIN(ts) as earliest,
       MAX(ts) as latest
FROM energy_new;
"

# View recent energy data
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT ts, consumption_total_w, consumption_l1_w 
FROM energy_new 
ORDER BY ts DESC 
LIMIT 10;
"
```

### Check Data Freshness

```bash
# Set up credentials
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Check if data is recent (within last 5 minutes)
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  'heatpump_new' as table_name,
  COUNT(*) as recent_rows,
  MAX(ts) as latest_timestamp,
  NOW() - MAX(ts) as age
FROM heatpump_new
WHERE ts > NOW() - INTERVAL '5 minutes'

UNION ALL

SELECT 
  'telemetry_new' as table_name,
  COUNT(*) as recent_rows,
  MAX(ts) as latest_timestamp,
  NOW() - MAX(ts) as age
FROM telemetry_new
WHERE ts > NOW() - INTERVAL '5 minutes'

UNION ALL

SELECT 
  'energy_new' as table_name,
  COUNT(*) as recent_rows,
  MAX(ts) as latest_timestamp,
  NOW() - MAX(ts) as age
FROM energy_new
WHERE ts > NOW() - INTERVAL '5 minutes';
"
```

## Step 4: Verify Redpanda Topics

### Check if Messages are in Redpanda

```bash
# List topics
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic list --brokers localhost:9092

# Check message count in topics
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic describe heatpump-realtime --brokers localhost:9092

kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic describe heatpump-telemetry --brokers localhost:9092

kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic describe energy-realtime --brokers localhost:9092
```

### Consume Messages from Redpanda (for testing)

```bash
# Consume a few messages from heatpump-realtime
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic consume heatpump-realtime --brokers localhost:9092 --num 5

# Consume from heatpump-telemetry
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic consume heatpump-telemetry --brokers localhost:9092 --num 5
```

## Step 5: Compare Old vs New Tables

### Check Both Systems are Writing

```bash
# Set up credentials
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Compare row counts
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  'heatpump (old)' as table_name,
  COUNT(*) as row_count,
  MAX(ts) as latest
FROM heatpump
UNION ALL
SELECT 
  'heatpump_new (new)' as table_name,
  COUNT(*) as row_count,
  MAX(ts) as latest
FROM heatpump_new;
"

# Compare recent data timestamps
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  'old' as source,
  COUNT(*) as rows_last_hour
FROM heatpump
WHERE ts > NOW() - INTERVAL '1 hour'
UNION ALL
SELECT 
  'new' as source,
  COUNT(*) as rows_last_hour
FROM heatpump_new
WHERE ts > NOW() - INTERVAL '1 hour';
"
```

## Step 6: Monitor Consumer Group

### Check Redpanda Consumer Group Status

```bash
# Check consumer group lag (if rpk supports it)
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk group describe redpanda-sink --brokers localhost:9092
```

## Troubleshooting

### No Data in Tables

1. **Check if migration ran successfully:**
   ```bash
   kubectl logs -n redpanda-sink job/redpanda-sink-migration
   ```

2. **Check if application is processing messages:**
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "processing\|batch"
   ```

3. **Verify Redpanda has messages:**
   ```bash
   kubectl exec -it redpanda-0 -n redpanda -- \
     rpk topic consume heatpump-realtime --brokers localhost:9092 --num 1
   ```

4. **Check database connection:**
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink | grep -i "database\|connected"
   ```

### Data Not Recent

1. **Check application logs for errors:**
   ```bash
   kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=50 | grep -i error
   ```

2. **Verify mqtt-input is publishing:**
   ```bash
   kubectl logs -n mqtt-input deployment/mqtt-input | grep -i "published\|sent"
   ```

3. **Check Redpanda topic offsets:**
   ```bash
   kubectl exec -it redpanda-0 -n redpanda -- \
     rpk topic describe heatpump-realtime --brokers localhost:9092
   ```

## Quick Verification Script

Save this as `verify-redpanda-sink.sh`:

```bash
#!/bin/bash

echo "=== Checking redpanda-sink Status ==="
kubectl get pods -n redpanda-sink -l app=redpanda-sink

echo -e "\n=== Recent Application Logs ==="
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=20

echo -e "\n=== Database Row Counts ==="
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
  'heatpump_new' as table_name,
  COUNT(*) as rows,
  MAX(ts) as latest
FROM heatpump_new
UNION ALL
SELECT 
  'telemetry_new' as table_name,
  COUNT(*) as rows,
  MAX(ts) as latest
FROM telemetry_new
UNION ALL
SELECT 
  'energy_new' as table_name,
  COUNT(*) as rows,
  MAX(ts) as latest
FROM energy_new;
"

echo -e "\n=== Redpanda Topic Status ==="
kubectl exec -it redpanda-0 -n redpanda -- \
  rpk topic list --brokers localhost:9092 | grep -E "heatpump|energy"
```

Make it executable and run:
```bash
chmod +x verify-redpanda-sink.sh
./verify-redpanda-sink.sh
```

