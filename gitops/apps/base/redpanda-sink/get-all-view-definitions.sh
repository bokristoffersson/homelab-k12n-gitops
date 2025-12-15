#!/bin/bash
# Script to get CREATE statements for all views listed
# Usage: ./get-all-view-definitions.sh

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get database credentials
if kubectl get secret timescaledb-secret -n redpanda-sink > /dev/null 2>&1; then
  SECRET_NS="redpanda-sink"
elif kubectl get secret timescaledb-secret -n heatpump-mqtt > /dev/null 2>&1; then
  SECRET_NS="heatpump-mqtt"
else
  echo "Error: Could not find timescaledb-secret"
  exit 1
fi

DB_USER=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

if [ -z "$DB_NAME" ]; then
  DB_NAME="timescaledb"
fi

VIEWS=(
  "energy_daily_summary"
  "energy_daily_summary_continuous"
  "energy_hourly_summary"
  "energy_hourly_summary_continuous"
  "energy_monthly_summary"
  "energy_monthly_summary_continuous"
  "energy_yearly_summary"
  "energy_yearly_summary_continuous"
  "heatpump_daily_summary"
)

echo -e "${GREEN}=== Getting CREATE statements for all views ===${NC}\n"

for view in "${VIEWS[@]}"; do
  echo -e "${YELLOW}=== $view ===${NC}"
  
  # Check if it's a continuous aggregate
  IS_CONTINUOUS=$(kubectl exec timescaledb-0 -n heatpump-mqtt -- \
    env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -tAc \
    "SELECT EXISTS (SELECT 1 FROM timescaledb_information.continuous_aggregates WHERE view_name = '$view');" 2>/dev/null || echo "false")
  
  if [ "$IS_CONTINUOUS" = "t" ]; then
    # Get continuous aggregate definition
    kubectl exec timescaledb-0 -n heatpump-mqtt -- \
      env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
      SELECT 
        'CREATE MATERIALIZED VIEW ' || view_name || ' WITH (timescaledb.continuous) AS' || E'\n' ||
        view_definition || E'\n' ||
        'WITH NO DATA;' as create_statement
      FROM timescaledb_information.continuous_aggregates
      WHERE view_name = '$view';
    " 2>&1 | grep -v "NOTICE" | tail -n +2
    
    # Get refresh policy
    kubectl exec timescaledb-0 -n heatpump-mqtt -- \
      env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
      SELECT 
        'PERFORM add_continuous_aggregate_policy(''' || view_name || ''',' ||
        E'\n  start_offset => INTERVAL ''' || start_offset::text || ''',' ||
        E'\n  end_offset => INTERVAL ''' || end_offset::text || ''',' ||
        E'\n  schedule_interval => INTERVAL ''' || schedule_interval::text || ''',' ||
        E'\n  if_not_exists => TRUE' ||
        E'\n);' as policy_statement
      FROM timescaledb_information.jobs
      WHERE proc_name LIKE '%continuous_aggregate%' AND view_name = '$view';
    " 2>&1 | grep -v "NOTICE" | tail -n +2
  else
    # Get regular view definition
    kubectl exec timescaledb-0 -n heatpump-mqtt -- \
      env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
      SELECT 'CREATE OR REPLACE VIEW ' || '$view' || ' AS' || E'\n' ||
             pg_get_viewdef('$view'::regclass, true) || ';' as create_statement;
    " 2>&1 | grep -v "NOTICE" | tail -n +2
  fi
  
  echo ""
done
