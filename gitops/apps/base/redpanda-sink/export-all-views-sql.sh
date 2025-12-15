#!/bin/bash
# Script to export all views as CREATE statements ready for migration files
# Outputs SQL that can be directly added to migration files

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Exporting All Views as CREATE Statements ===${NC}\n"

# Get database credentials from Kubernetes secret
DB_USER=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n redpanda-sink -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

if [ -z "$DB_USER" ] || [ -z "$DB_PASSWORD" ] || [ -z "$DB_NAME" ]; then
  echo "Error: Could not retrieve database credentials from Kubernetes secret"
  exit 1
fi

OUTPUT_FILE="exported_views_$(date +%Y%m%d_%H%M%S).sql"

# Create SQL query
SQL_QUERY=$(cat <<'EOF'
-- Export all views, materialized views, and continuous aggregates as CREATE statements

\echo '-- ============================================'
\echo '-- Exported Views and Aggregates'
\echo '-- Generated: ' || CURRENT_TIMESTAMP
\echo '-- ============================================'
\echo ''
\echo '-- Regular Views'
\echo '-- ============================================'
\echo ''

DO $$
DECLARE
    v_rec RECORD;
    v_def TEXT;
BEGIN
    FOR v_rec IN 
        SELECT viewname, definition
        FROM pg_views
        WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
        ORDER BY viewname
    LOOP
        RAISE NOTICE E'\n-- View: %\nCREATE OR REPLACE VIEW % AS\n%;\n', 
            v_rec.viewname, v_rec.viewname, v_rec.definition;
    END LOOP;
END $$;

\echo ''
\echo '-- Materialized Views (Non-Continuous)'
\echo '-- ============================================'
\echo ''

DO $$
DECLARE
    v_rec RECORD;
    v_def TEXT;
    v_continuous BOOLEAN;
BEGIN
    FOR v_rec IN 
        SELECT matviewname, definition
        FROM pg_matviews
        WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
        ORDER BY matviewname
    LOOP
        -- Check if it's a continuous aggregate
        SELECT EXISTS (
            SELECT 1 FROM timescaledb_information.continuous_aggregates 
            WHERE view_name = v_rec.matviewname
        ) INTO v_continuous;
        
        IF NOT v_continuous THEN
            RAISE NOTICE E'\n-- Materialized View: %\nCREATE MATERIALIZED VIEW IF NOT EXISTS % AS\n%;\n', 
                v_rec.matviewname, v_rec.matviewname, v_rec.definition;
        END IF;
    END LOOP;
END $$;

\echo ''
\echo '-- TimescaleDB Continuous Aggregates'
\echo '-- ============================================'
\echo ''

DO $$
DECLARE
    v_rec RECORD;
    v_def TEXT;
    v_policy RECORD;
BEGIN
    FOR v_rec IN 
        SELECT 
            view_name,
            view_definition,
            materialized_only,
            finalized
        FROM timescaledb_information.continuous_aggregates
        ORDER BY view_name
    LOOP
        RAISE NOTICE E'\n-- Continuous Aggregate: %\n', v_rec.view_name;
        RAISE NOTICE '-- Note: Continuous aggregates require special syntax.';
        RAISE NOTICE '-- The view_definition below shows the SELECT, but you need:';
        RAISE NOTICE '-- CREATE MATERIALIZED VIEW % WITH (timescaledb.continuous) AS', v_rec.view_name;
        RAISE NOTICE '%', v_rec.view_definition;
        RAISE NOTICE '-- WITH NO DATA;';
        RAISE NOTICE '';
        
        -- Get refresh policies
        FOR v_policy IN 
            SELECT 
                policy_name,
                start_offset,
                end_offset,
                schedule_interval,
                timezone
            FROM timescaledb_information.jobs
            WHERE proc_name LIKE '%continuous_aggregate%'
              AND view_name = v_rec.view_name
        LOOP
            RAISE NOTICE '-- Refresh Policy:';
            RAISE NOTICE '-- PERFORM add_continuous_aggregate_policy(''%'',', v_rec.view_name;
            RAISE NOTICE '--   start_offset => INTERVAL ''%'',', v_policy.start_offset;
            RAISE NOTICE '--   end_offset => INTERVAL ''%'',', v_policy.end_offset;
            RAISE NOTICE '--   schedule_interval => INTERVAL ''%'',', v_policy.schedule_interval;
            RAISE NOTICE '--   if_not_exists => TRUE';
            RAISE NOTICE '-- );';
            RAISE NOTICE '';
        END LOOP;
    END LOOP;
END $$;

\echo ''
\echo '-- Indexes on Materialized Views'
\echo '-- ============================================'
\echo ''

DO $$
DECLARE
    v_rec RECORD;
BEGIN
    FOR v_rec IN 
        SELECT 
            tablename,
            indexname,
            indexdef
        FROM pg_indexes
        WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
          AND tablename IN (
            SELECT matviewname 
            FROM pg_matviews 
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
          )
        ORDER BY tablename, indexname
    LOOP
        RAISE NOTICE '-- Index on %: %', v_rec.tablename, v_rec.indexname;
        RAISE NOTICE '%;', v_rec.indexdef;
        RAISE NOTICE '';
    END LOOP;
END $$;
EOF
)

echo "Querying database and generating SQL..."
echo ""

# Execute and save to file
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "$SQL_QUERY" 2>&1 | \
  grep -v "^NOTICE:" | \
  sed 's/^NOTICE:  //' > "$OUTPUT_FILE"

echo -e "${GREEN}✓${NC} SQL exported to: ${BLUE}$OUTPUT_FILE${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Review the exported SQL file: $OUTPUT_FILE"
echo "2. Compare with existing migrations to identify missing views"
echo "3. Create new migration files for views that don't have migrations yet"
echo "4. Use the existing migration 005_add_energy_continuous_aggregates.sql as a template"
echo ""
