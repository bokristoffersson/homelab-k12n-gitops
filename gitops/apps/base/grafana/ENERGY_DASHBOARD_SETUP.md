# Energy Dashboard Setup

This document describes how to set up the Energy Dashboard in Grafana.

## Overview

The Energy Dashboard displays hourly energy consumption data from TimescaleDB. The dashboard uses the `energy_hourly_summary` materialized view to show energy consumption patterns throughout the day.

## Prerequisites

1. TimescaleDB is running in the `heatpump-mqtt` namespace
2. The `energy` table exists with data
3. The `energy_hourly_summary` materialized view is created

## Setup Steps

### 1. Configure TimescaleDB Datasource in Grafana

Since the credentials are stored in a SealedSecret in the `heatpump-mqtt` namespace, you'll need to manually configure the datasource in Grafana UI:

1. Log into Grafana at https://grafana.k12n.com
2. Go to Configuration → Data Sources
3. Click "Add data source"
4. Select "PostgreSQL"
5. Configure:
   - **Name**: TimescaleDB
   - **Host**: `timescaledb.heatpump-mqtt.svc.cluster.local:5432`
   - **Database**: `heatpump`
   - **User**: Get from the timescaledb-secret in heatpump-mqtt namespace
   - **Password**: Get from the timescaledb-secret in heatpump-mqtt namespace
   - **SSL Mode**: Disable
   - **TimescaleDB**: Check the box
6. Click "Save & Test"

To get the credentials:

```bash
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d
kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d
```

### 2. Import the Dashboard

The dashboard is automatically provisioned from the ConfigMap. It should appear in Grafana under Dashboards.

If it doesn't appear automatically:

1. Go to Dashboards → Import
2. Copy the contents of `dashboard-energy-configmap.yaml` (the JSON inside the `data` section)
3. Paste it and click "Load"
4. Select the TimescaleDB datasource
5. Click "Import"

### 3. Dashboard Features

The Energy Dashboard includes:

- **Hourly Energy Consumption Chart**: Bar chart showing energy consumption by hour of day
- **Time Range**: Default shows last 7 days
- **Auto-refresh**: Every 30 seconds

## Query Details

The dashboard uses the following SQL query to retrieve hourly energy data:

```sql
SELECT
  EXTRACT(HOUR FROM hour) AS "hour",
  hourly_energy_consumption_total AS "energy_wh"
FROM energy_hourly_summary
WHERE $__timeFilter(hour)
ORDER BY hour
```

This query extracts the hour (0-23) from the timestamp and shows the energy consumed during that hour in Watt-hours.

## Troubleshooting

### Dashboard shows "No data"

1. Verify TimescaleDB is running: `kubectl get pods -n heatpump-mqtt`
2. Verify the energy table has data:
   ```bash
   kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U <user> -d heatpump -c "SELECT COUNT(*) FROM energy_hourly_summary;"
   ```
3. Check datasource connection in Grafana
4. Verify the materialized view exists:
   ```bash
   kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U <user> -d heatpump -c "\d energy_hourly_summary"
   ```

### Datasource connection fails

1. Check network connectivity: the Grafana pod can reach the TimescaleDB service
2. Verify credentials are correct
3. Check firewall/network policies

## Maintenance

The materialized view `energy_hourly_summary` is automatically maintained by TimescaleDB. If you need to refresh it manually:

```bash
kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U <user> -d heatpump -c "CALL refresh_continuous_aggregate('energy_hourly_summary', NULL, NULL);"
```

