# Grafana "No Data" Troubleshooting Guide

## Problem Summary
The dashboards show "No Data" because the Grafana datasource configuration is missing valid database credentials.

## Issues Found

1. **Missing Credentials**: The datasource needs valid database credentials (user and password). The current configuration has placeholders that need to be replaced.
2. **Database Connection**: The datasource is configured to connect to database `timescaledb` which contains the `heatpump` table and the `energy_*_summary` materialized views.

## Solutions

### Option 1: Update the GitOps Configuration (Recommended)

1. **Get the database credentials** from your sealed secret:
   ```bash
   kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_USER}' | base64 -d
   kubectl get secret timescaledb-secret -n heatpump-mqtt -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d
   ```

2. **Seal new credentials** for Grafana:
   ```bash
   # Create a temporary secret
   kubectl create secret generic grafana-db-credentials \
     --from-literal=user='YOUR_DB_USER' \
     --from-literal=password='YOUR_DB_PASSWORD' \
     -n grafana \
     --dry-run=client -o yaml > /tmp/grafana-db-credentials.yaml
   
   # Seal it
   kubectl seal < /tmp/grafana-db-credentials.yaml -w apps/base/grafana/grafana-db-credentials-sealed.yaml
   
   # Clean up
   rm /tmp/grafana-db-credentials.yaml
   ```

3. **Update the HelmRelease** to reference the credentials properly. The current configuration has placeholders that need to be replaced.

4. **Apply and reconcile**:
   ```bash
   flux reconcile helmrelease grafana -n grafana
   ```

### Option 2: Manual Configuration in Grafana UI

1. Log into Grafana
2. Go to Configuration → Data Sources
3. Find the "TimescaleDB" datasource
4. Update:
   - Database name: `timescaledb`
   - User: (get from the secret above)
   - Password: (get from the secret above)
5. Click "Save & Test"

## Verification Steps

After fixing the configuration:

1. **Check datasource connectivity**:
   - Go to Grafana UI → Data Sources
   - Click on "TimescaleDB"
   - Click "Save & Test"
   - Should see "Data source is working"

2. **Verify data exists**:
   ```bash
   kubectl exec -it timescaledb-0 -n heatpump-mqtt -- psql -U YOUR_USER -d timescaledb -c "SELECT COUNT(*) FROM energy_hourly_summary LIMIT 10;"
   ```

3. **Check dashboard queries**:
   - Edit a panel in the dashboard
   - Go to the Query Inspector (third icon on bottom)
   - Run the query and check for errors

## Common Issues

### "relation 'energy_hourly_summary' does not exist"
- The materialized views haven't been created yet
- Run `energy_table.sql` in the database

### "database does not exist"
- Check the database name is `heatpump`
- Verify the StatefulSet is running: `kubectl get statefulset -n heatpump-mqtt`

### "Connection refused"
- Verify the Service is running: `kubectl get svc -n heatpump-mqtt`
- Check network policies allow traffic from `grafana` namespace

## Database Schema Reference

The dashboard expects these tables/views to exist:
- `energy` - Raw energy data
- `energy_hourly_summary` - Hourly aggregated data
- `energy_daily_summary` - Daily aggregated data  
- `energy_monthly_summary` - Monthly aggregated data
- `energy_yearly_summary` - Yearly aggregated data

These should be created by the `energy_table.sql` script.

