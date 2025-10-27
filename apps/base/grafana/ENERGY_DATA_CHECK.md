# Energy Data Diagnostic

## Quick Check: Verify Data Exists

Run this query in the Grafana Explore view to check if your data exists:

### Check 1: Raw energy table
```sql
SELECT 
  COUNT(*) as "total_rows",
  MIN(ts) as "first_record",
  MAX(ts) as "last_record"
FROM energy;
```

### Check 2: Materialized view data
```sql
SELECT 
  COUNT(*) as "total_hours",
  MIN(hour) as "earliest_hour",
  MAX(hour) as "latest_hour"
FROM energy_hourly_summary;
```

### Check 3: Recent data (last 2 hours)
```sql
SELECT 
  ts,
  consumption_total_w
FROM energy
WHERE ts > NOW() - INTERVAL '2 hours'
ORDER BY ts DESC
LIMIT 10;
```

## Dashboard Options

### Option A: Real-time Dashboard (for data < 24 hours old)
Import `energy-dashboard-realtime.json` - this queries the raw `energy` table and shows data from the last 3 hours.

### Option B: Hourly Summary Dashboard (for older data)
Import `energy-dashboard-import.json` - this uses the materialized view `energy_hourly_summary`.

## Common Issues

### 1. Materialized View not Refreshed
If you have raw data but the materialized view is empty, refresh it:
```bash
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U <user> -d heatpump -c "CALL refresh_continuous_aggregate('energy_hourly_summary', NULL, NULL);"
```

### 2. Materialized View Not Created
If the view doesn't exist, run the SQL from `energy_table.sql`:
```bash
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U <user> -d heatpump -f /path/to/energy_table.sql
```

### 3. Wrong Database Name
Verify you're connected to the right database. Check:
- In Grafana datasource: database should be `heatpump`
- Connection should be: `timescaledb.heatpump-mqtt.svc.cluster.local:5432`

## Time Range Settings

- For **real-time data** (< 24 hours): Use "now-6h" to "now"
- For **hourly summaries**: Use "now-7d" to "now"
- For **testing**: Try "now-2h" to "now"

