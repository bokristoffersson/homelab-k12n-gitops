# Checking Database for 500 Errors

The 500 errors are likely due to missing database objects (tables, views, or materialized views).

## Quick Check: Test API with Detailed Error

1. **Get your JWT token** from browser localStorage
2. **Test an endpoint** and see the actual error:

```bash
# Replace YOUR_TOKEN with your actual JWT token
TOKEN="YOUR_TOKEN"

# Test energy latest endpoint
curl -v -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/energy/latest 2>&1 | grep -A 20 "< HTTP"
```

The response body should contain the actual error message from the backend.

## Check Database Objects

### 1. Check if Tables Exist

```bash
# Connect to database pod
kubectl exec -it -n heatpump-mqtt $(kubectl get pod -n heatpump-mqtt -l app=timescaledb -o jsonpath='{.items[0].metadata.name}') -- psql -U timescale -d timescaledb

# Then run:
\dt
\dv
\dmv  # materialized views
```

### 2. Check if Required Views Exist

The API likely needs these views:
- `energy_hourly` - continuous aggregate for hourly energy data
- Views for `heatpump` latest data
- Views for `energy` latest data

### 3. Check if Migrations Ran

```bash
# Check migration jobs
kubectl get jobs -n heatpump-mqtt
kubectl get jobs -n redpanda-sink

# Check if migration job completed successfully
kubectl logs -n <namespace> job/<migration-job-name>
```

## Common Issues

### Issue: Missing Materialized Views

If `energy_hourly` or other views don't exist:

1. **Check migration files** in `gitops/apps/base/redpanda-sink/migrations/`
2. **Run migrations manually** if needed
3. **Check if continuous aggregates are created**

### Issue: Empty Database

If tables exist but are empty:

1. Check if data is being ingested from Redpanda
2. Check Redpanda topics have data
3. Check backend logs for ingestion errors

### Issue: Wrong Table/View Names

The API might be querying tables/views with different names than what exists.

## Next Steps

1. **Check browser Network tab** - Look at the actual error response body
2. **Check backend logs in real-time** while making a request:
   ```bash
   kubectl logs -n redpanda-sink -l app=redpanda-sink -f
   ```
3. **Test API directly** with curl to see the error message
4. **Check database** to see what objects exist

## Debugging Commands

```bash
# Watch backend logs while making a request
kubectl logs -n redpanda-sink -l app=redpanda-sink -f

# In another terminal, make a request
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/energy/latest

# Check what tables/views exist
kubectl exec -n heatpump-mqtt -it <timescaledb-pod> -- \
  psql -U timescale -d timescaledb -c "\dt"
kubectl exec -n heatpump-mqtt -it <timescaledb-pod> -- \
  psql -U timescale -d timescaledb -c "\dmv"
```
