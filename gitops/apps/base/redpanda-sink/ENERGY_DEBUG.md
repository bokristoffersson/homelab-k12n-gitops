# Debugging Energy Sensor Data Flow

## Quick Debug Commands

### 1. Check mqtt-input is receiving messages
```bash
# Check recent logs
kubectl logs -n mqtt-input deployment/mqtt-input --tail=100 | grep -i "energy\|saveeye"

# Follow logs in real-time
kubectl logs -n mqtt-input deployment/mqtt-input -f | grep -i "energy\|saveeye"
```

### 2. Check if messages are in Redpanda
```bash
# List topics
kubectl exec -it redpanda-0 -n redpanda -- rpk topic list --brokers localhost:9092

# Describe energy-realtime topic
kubectl exec -it redpanda-0 -n redpanda -- rpk topic describe energy-realtime --brokers localhost:9092

# Consume messages (wait up to 10 seconds)
timeout 10 kubectl exec -it redpanda-0 -n redpanda -- rpk topic consume energy-realtime --brokers localhost:9092 --num 5
```

### 3. Check redpanda-sink is processing
```bash
# Check recent logs
kubectl logs -n redpanda-sink deployment/redpanda-sink --tail=100 | grep -i "energy"

# Follow logs in real-time
kubectl logs -n redpanda-sink deployment/redpanda-sink -f | grep -i "energy"
```

### 4. Check database
```bash
# Get credentials
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Check row count
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c \
  "SELECT COUNT(*) as row_count, MIN(ts) as earliest, MAX(ts) as latest FROM energy_new;"

# View recent data
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c \
  "SELECT * FROM energy_new ORDER BY ts DESC LIMIT 10;"
```

## Common Issues

### Issue 1: mqtt-input not receiving MQTT messages
**Symptoms**: No logs showing "saveeye/telemetry" messages

**Check**:
- Verify MQTT broker is running: `kubectl get pods -n mosquitto`
- Verify topic exists and has messages (use MQTT client to subscribe)
- Check mqtt-input ConfigMap has correct MQTT credentials
- Verify mqtt-input pod is running: `kubectl get pods -n mqtt-input`

### Issue 2: Messages not reaching Redpanda
**Symptoms**: mqtt-input logs show messages but Redpanda topic is empty

**Check**:
- Look for publishing errors in mqtt-input logs
- Verify Redpanda is accessible: `kubectl get pods -n redpanda`
- Check mqtt-input ConfigMap has correct Redpanda broker address
- Verify energy-realtime topic exists

### Issue 3: redpanda-sink not processing messages
**Symptoms**: Messages in Redpanda but no database writes

**Check**:
- Look for errors in redpanda-sink logs
- Verify consumer group: `kubectl exec -it redpanda-0 -n redpanda -- rpk group describe redpanda-sink --brokers localhost:9092`
- Check redpanda-sink ConfigMap has correct topic name: "energy-realtime"
- Verify redpanda-sink pod is running: `kubectl get pods -n redpanda-sink`

### Issue 4: Message format mismatch
**Symptoms**: Processing errors in logs about field extraction

**Check**:
- Verify actual MQTT message format matches expected structure
- Check if `activeTotalConsumption` field exists in source messages
- Verify nested structure: `{"activeTotalConsumption": {"total": ..., "L1": ..., "L2": ..., "L3": ...}}`

## Expected Message Flow

1. **MQTT Message** (saveeye/telemetry):
   ```json
   {
     "timestamp": "2025-01-XX...",
     "activeTotalConsumption": {
       "total": 1000,
       "L1": 400,
       "L2": 300,
       "L3": 300
     }
   }
   ```

2. **mqtt-input publishes to Redpanda** (energy-realtime):
   ```json
   {
     "ts": "2025-01-XX...",
     "fields": {
       "consumption_total_w": 1000,
       "consumption_l1_w": 400,
       "consumption_l2_w": 300,
       "consumption_l3_w": 300
     }
   }
   ```

3. **redpanda-sink writes to database** (energy_new table):
   - ts: timestamp
   - consumption_total_w: 1000
   - consumption_l1_w: 400
   - consumption_l2_w: 300
   - consumption_l3_w: 300

## Verify Configuration

### mqtt-input ConfigMap
```bash
kubectl get configmap -n mqtt-input mqtt-input-config -o yaml | grep -A 20 "name: \"energy\""
```

Should show:
- topic: "saveeye/telemetry"
- redpanda_topic: "energy-realtime"
- fields.activeTotalConsumption with nested attributes

### redpanda-sink ConfigMap
```bash
kubectl get configmap -n redpanda-sink redpanda-sink-config -o yaml | grep -A 20 "name: \"energy\""
```

Should show:
- topic: "energy-realtime"
- table: "energy_new"
- timestamp path: "$.ts", format: "rfc3339"
- fields: consumption_total_w, consumption_l1_w, consumption_l2_w, consumption_l3_w

## Test Message Format

To verify the actual MQTT message format, you can:
1. Subscribe to the MQTT topic using an MQTT client
2. Or check the old heatpump-mqtt logs to see what format it was receiving
3. Or check if there are any sample messages in the old energy table

```bash
# Check old energy table structure (if it has data)
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c \
  "SELECT * FROM energy ORDER BY ts DESC LIMIT 5;"
```

