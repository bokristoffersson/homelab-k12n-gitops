#!/bin/bash
# Script to get the full CREATE statement for a specific view
# Usage: ./get-view-definition.sh <view_name>

set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <view_name>"
  echo ""
  echo "Example: $0 energy_hourly_summary"
  exit 1
fi

VIEW_NAME="$1"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Getting CREATE statement for: $VIEW_NAME ===${NC}\n"

# Get database credentials from Kubernetes secret
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

if [ -z "$DB_USER" ] || [ -z "$DB_PASSWORD" ] || [ -z "$DB_NAME" ]; then
  echo "Error: Could not retrieve database credentials from Kubernetes secret"
  exit 1
fi

# Create SQL to get the view definition
SQL_QUERY=$(cat <<EOF
-- Check if it's a regular view
DO \$\$
DECLARE
    v_def TEXT;
    v_is_materialized BOOLEAN;
    v_is_continuous BOOLEAN;
BEGIN
    -- Check if it's a continuous aggregate
    SELECT EXISTS (
        SELECT 1 FROM timescaledb_information.continuous_aggregates 
        WHERE view_name = '$VIEW_NAME'
    ) INTO v_is_continuous;
    
    -- Check if it's a materialized view
    SELECT EXISTS (
        SELECT 1 FROM pg_matviews 
        WHERE matviewname = '$VIEW_NAME'
    ) INTO v_is_materialized;
    
    IF v_is_continuous THEN
        RAISE NOTICE '=== CONTINUOUS AGGREGATE: $VIEW_NAME ===';
        -- Get the view definition from continuous aggregates
        SELECT view_definition INTO v_def
        FROM timescaledb_information.continuous_aggregates
        WHERE view_name = '$VIEW_NAME';
        
        RAISE NOTICE 'Definition:';
        RAISE NOTICE '%', v_def;
        
        -- Get refresh policy info
        RAISE NOTICE '';
        RAISE NOTICE '=== REFRESH POLICY ===';
        FOR v_def IN 
            SELECT 
                'Policy: ' || policy_name || E'\n' ||
                '  Start Offset: ' || start_offset::text || E'\n' ||
                '  End Offset: ' || end_offset::text || E'\n' ||
                '  Schedule Interval: ' || schedule_interval::text || E'\n' ||
                '  Timezone: ' || COALESCE(timezone, 'NULL')
            FROM timescaledb_information.jobs
            WHERE proc_name LIKE '%continuous_aggregate%'
              AND view_name = '$VIEW_NAME'
        LOOP
            RAISE NOTICE '%', v_def;
        END LOOP;
        
    ELSIF v_is_materialized THEN
        RAISE NOTICE '=== MATERIALIZED VIEW: $VIEW_NAME ===';
        SELECT definition INTO v_def
        FROM pg_matviews
        WHERE matviewname = '$VIEW_NAME';
        
        RAISE NOTICE 'Definition:';
        RAISE NOTICE '%', v_def;
        
    ELSE
        -- Regular view
        RAISE NOTICE '=== REGULAR VIEW: $VIEW_NAME ===';
        SELECT definition INTO v_def
        FROM pg_views
        WHERE viewname = '$VIEW_NAME';
        
        IF v_def IS NULL THEN
            RAISE NOTICE 'View not found!';
        ELSE
            RAISE NOTICE 'Definition:';
            RAISE NOTICE '%', v_def;
        END IF;
    END IF;
    
    -- Get indexes on the view
    RAISE NOTICE '';
    RAISE NOTICE '=== INDEXES ===';
    FOR v_def IN 
        SELECT indexdef
        FROM pg_indexes
        WHERE tablename = '$VIEW_NAME'
          AND schemaname NOT IN ('pg_catalog', 'information_schema')
    LOOP
        RAISE NOTICE '%', v_def;
    END LOOP;
    
END \$\$;
EOF
)

# Execute the query
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "$SQL_QUERY"

echo ""
echo -e "${GREEN}=== Complete ===${NC}"
echo ""
echo -e "${YELLOW}To create a migration from this:${NC}"
echo "1. Copy the definition above"
echo "2. Create a new migration file: migrations/00X_add_${VIEW_NAME}.sql"
echo "3. Wrap it in appropriate IF EXISTS checks (see 005_add_energy_continuous_aggregates.sql for example)"
