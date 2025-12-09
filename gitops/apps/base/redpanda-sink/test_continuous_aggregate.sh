#!/bin/bash
# Test script to verify the energy_hourly continuous aggregate exists and works

set -e

echo "=== Testing Continuous Aggregate: energy_hourly ==="
echo ""

# Step 1: Check migration job status
echo "1. Checking migration job status..."
kubectl get jobs -n redpanda-sink -l migration-version=005
echo ""

# Step 2: Get database credentials
echo "2. Getting database credentials..."
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

echo "Database: $DB_NAME"
echo "User: $DB_USER"
echo ""

# Step 3: Check if continuous aggregate exists
echo "3. Checking if continuous aggregate 'energy_hourly' exists..."
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  psql -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT 
      view_name,
      materialized_only,
      finalized
    FROM timescaledb_information.continuous_aggregates
    WHERE view_name = 'energy_hourly';
  "
echo ""

# Step 4: Check continuous aggregate definition
echo "4. Checking continuous aggregate definition..."
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  psql -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT 
      view_definition
    FROM timescaledb_information.continuous_aggregates
    WHERE view_name = 'energy_hourly';
  "
echo ""

# Step 5: Check refresh policy
echo "5. Checking refresh policy..."
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  psql -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT 
      view_name,
      schedule_interval,
      start_offset,
      end_offset
    FROM timescaledb_information.jobs
    WHERE proc_name = 'policy_refresh_continuous_aggregate'
    AND config::jsonb->>'mat_hypertable_id' IN (
      SELECT materialization_hypertable_id 
      FROM timescaledb_information.continuous_aggregates 
      WHERE view_name = 'energy_hourly'
    );
  "
echo ""

# Step 6: Query the continuous aggregate (if data exists)
echo "6. Querying continuous aggregate for recent data..."
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  psql -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT 
      hour_start,
      hour_end,
      total_energy_kwh,
      total_energy_l1_kwh,
      total_energy_l2_kwh,
      total_energy_l3_kwh,
      measurement_count
    FROM energy_hourly
    ORDER BY hour_start DESC
    LIMIT 10;
  "
echo ""

# Step 7: Check if underlying energy table has data
echo "7. Checking underlying energy table for recent data..."
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  psql -U "$DB_USER" -d "$DB_NAME" -c "
    SELECT 
      COUNT(*) as total_rows,
      MIN(ts) as earliest_reading,
      MAX(ts) as latest_reading
    FROM energy;
  "
echo ""

# Step 8: Manually refresh the continuous aggregate (optional)
echo "8. To manually refresh the continuous aggregate, run:"
echo "   kubectl exec timescaledb-0 -n heatpump-mqtt -- \\"
echo "     psql -U \"$DB_USER\" -d \"$DB_NAME\" -c \"CALL refresh_continuous_aggregate('energy_hourly', NULL, NULL);\""
echo ""

echo "=== Test Complete ==="

