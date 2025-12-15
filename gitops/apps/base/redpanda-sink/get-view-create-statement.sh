#!/bin/bash
# Script to get the full CREATE statement for a specific view using pg_get_viewdef
# Usage: ./get-view-create-statement.sh <view_name> [schema]

set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <view_name> [schema]"
  echo ""
  echo "Example: $0 energy_hourly_summary"
  echo "Example: $0 energy_hourly_summary public"
  exit 1
fi

VIEW_NAME="$1"
SCHEMA_NAME="${2:-public}"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Getting CREATE statement for: ${SCHEMA_NAME}.${VIEW_NAME} ===${NC}\n"

# Get database credentials from Kubernetes secret
# Try redpanda-sink namespace first, then heatpump-mqtt
if kubectl get secret timescaledb-secret -n redpanda-sink > /dev/null 2>&1; then
  SECRET_NS="redpanda-sink"
elif kubectl get secret timescaledb-secret -n heatpump-mqtt > /dev/null 2>&1; then
  SECRET_NS="heatpump-mqtt"
else
  echo "Error: Could not find timescaledb-secret in redpanda-sink or heatpump-mqtt namespaces"
  exit 1
fi

DB_USER=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Fallback to hardcoded database name if secret doesn't have it
if [ -z "$DB_NAME" ]; then
  DB_NAME="timescaledb"
fi

if [ -z "$DB_USER" ] || [ -z "$DB_PASSWORD" ]; then
  echo "Error: Could not retrieve database credentials from Kubernetes secret"
  exit 1
fi

# Create SQL query
SQL_QUERY=$(cat <<EOF
DO \$\$
DECLARE
    v_def TEXT;
    v_is_materialized BOOLEAN;
    v_is_continuous BOOLEAN;
    v_schema TEXT := '$SCHEMA_NAME';
    v_view TEXT := '$VIEW_NAME';
    v_full_name TEXT;
BEGIN
    -- Try to find the view in any schema if not found in specified schema
    IF NOT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = v_schema AND c.relname = v_view
    ) THEN
        -- Search all schemas
        SELECT n.nspname, c.relname INTO v_schema, v_view
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relname = '$VIEW_NAME'
          AND n.nspname NOT IN ('pg_catalog', 'information_schema')
        LIMIT 1;
        
        IF v_view IS NULL THEN
            RAISE EXCEPTION 'View % not found in any schema', '$VIEW_NAME';
        END IF;
    END IF;
    
    v_full_name := v_schema || '.' || v_view;
    RAISE NOTICE 'Found view: %', v_full_name;
    
    -- Check if it's a continuous aggregate
    SELECT EXISTS (
        SELECT 1 FROM timescaledb_information.continuous_aggregates 
        WHERE view_name = v_view
    ) INTO v_is_continuous;
    
    -- Check if it's a materialized view
    SELECT EXISTS (
        SELECT 1 FROM pg_matviews 
        WHERE schemaname = v_schema AND matviewname = v_view
    ) INTO v_is_materialized;
    
    RAISE NOTICE '';
    
    IF v_is_continuous THEN
        RAISE NOTICE '=== CONTINUOUS AGGREGATE ===';
        RAISE NOTICE '';
        RAISE NOTICE 'CREATE MATERIALIZED VIEW % WITH (timescaledb.continuous) AS', v_view;
        
        -- Get the view definition
        SELECT view_definition INTO v_def
        FROM timescaledb_information.continuous_aggregates
        WHERE view_name = v_view;
        
        RAISE NOTICE '%', v_def;
        RAISE NOTICE 'WITH NO DATA;';
        RAISE NOTICE '';
        
        -- Get refresh policy
        FOR v_def IN 
            SELECT 
                'PERFORM add_continuous_aggregate_policy(''' || v_view || ''',' ||
                E'\n  start_offset => INTERVAL ''' || start_offset::text || ''',' ||
                E'\n  end_offset => INTERVAL ''' || end_offset::text || ''',' ||
                E'\n  schedule_interval => INTERVAL ''' || schedule_interval::text || ''',' ||
                E'\n  if_not_exists => TRUE' ||
                E'\n);'
            FROM timescaledb_information.jobs
            WHERE proc_name LIKE '%continuous_aggregate%'
              AND view_name = v_view
        LOOP
            RAISE NOTICE '-- Refresh Policy:';
            RAISE NOTICE '%', v_def;
        END LOOP;
        
    ELSIF v_is_materialized THEN
        RAISE NOTICE '=== MATERIALIZED VIEW ===';
        RAISE NOTICE '';
        RAISE NOTICE 'CREATE MATERIALIZED VIEW % AS', v_view;
        
        -- Use pg_get_viewdef to get the actual definition
        BEGIN
            SELECT pg_get_viewdef(v_full_name::regclass, true) INTO v_def;
            RAISE NOTICE '%', v_def;
            RAISE NOTICE ';';
        EXCEPTION WHEN OTHERS THEN
            -- Fallback to pg_matviews
            SELECT definition INTO v_def
            FROM pg_matviews
            WHERE schemaname = v_schema AND matviewname = v_view;
            RAISE NOTICE '%', v_def;
            RAISE NOTICE ';';
        END;
        
    ELSE
        -- Regular view
        RAISE NOTICE '=== REGULAR VIEW ===';
        RAISE NOTICE '';
        RAISE NOTICE 'CREATE OR REPLACE VIEW % AS', v_view;
        
        -- Use pg_get_viewdef
        BEGIN
            SELECT pg_get_viewdef(v_full_name::regclass, true) INTO v_def;
            RAISE NOTICE '%', v_def;
            RAISE NOTICE ';';
        EXCEPTION WHEN OTHERS THEN
            -- Fallback to pg_views
            SELECT definition INTO v_def
            FROM pg_views
            WHERE schemaname = v_schema AND viewname = v_view;
            IF v_def IS NULL THEN
                RAISE EXCEPTION 'Could not get definition for view %', v_full_name;
            END IF;
            RAISE NOTICE '%', v_def;
            RAISE NOTICE ';';
        END;
    END IF;
    
    -- Get indexes
    RAISE NOTICE '';
    RAISE NOTICE '=== INDEXES ===';
    FOR v_def IN 
        SELECT indexdef || ';'
        FROM pg_indexes
        WHERE schemaname = v_schema
          AND tablename = v_view
    LOOP
        RAISE NOTICE '%', v_def;
    END LOOP;
    
END \$\$;
EOF
)

# Execute the query
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "$SQL_QUERY" 2>&1 | \
  grep -v "^DO" | \
  sed 's/^NOTICE:  //' | \
  sed '/^$/d'

echo ""
echo -e "${GREEN}=== Complete ===${NC}"
