#!/bin/bash
# Script to extract all aggregate views from the database
# This helps identify manually created views that need to be added to migrations

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Extracting Aggregate Views from Database ===${NC}\n"

# Get database credentials from Kubernetes secret
# Try redpanda-sink namespace first, then heatpump-mqtt
echo "Fetching database credentials..."
if kubectl get secret timescaledb-secret -n redpanda-sink > /dev/null 2>&1; then
  SECRET_NS="redpanda-sink"
elif kubectl get secret timescaledb-secret -n heatpump-mqtt > /dev/null 2>&1; then
  SECRET_NS="heatpump-mqtt"
else
  echo -e "${RED}Error: Could not find timescaledb-secret in redpanda-sink or heatpump-mqtt namespaces${NC}"
  exit 1
fi

echo -e "${BLUE}Using secret from namespace: $SECRET_NS${NC}"

DB_USER=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_USER}' | base64 -d)
DB_PASSWORD=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_PASSWORD}' | base64 -d)
DB_NAME=$(kubectl get secret timescaledb-secret -n "$SECRET_NS" -o jsonpath='{.data.POSTGRES_DB}' | base64 -d)

# Fallback to hardcoded database name if secret doesn't have it
if [ -z "$DB_NAME" ]; then
  DB_NAME="timescaledb"
  echo -e "${YELLOW}Warning: POSTGRES_DB not in secret, using default: $DB_NAME${NC}"
fi

if [ -z "$DB_USER" ] || [ -z "$DB_PASSWORD" ]; then
  echo -e "${RED}Error: Could not retrieve database credentials from Kubernetes secret${NC}"
  exit 1
fi

echo -e "${GREEN}✓${NC} Database: $DB_NAME"
echo -e "${GREEN}✓${NC} User: $DB_USER"
echo ""

# Test connection first
echo "Testing database connection..."
if ! kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" > /dev/null 2>&1; then
  echo -e "${RED}Error: Could not connect to database${NC}"
  exit 1
fi
echo -e "${GREEN}✓${NC} Connection successful\n"

# Create SQL file with better queries
SQL_FILE=$(mktemp)
cat > "$SQL_FILE" << 'EOF'
-- First, let's see what schemas exist
\echo '=== Available Schemas ==='
SELECT nspname as schema_name 
FROM pg_namespace 
WHERE nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast', 'pg_temp_1', 'pg_toast_temp_1')
ORDER BY nspname;

\echo ''
\echo '=== ALL REGULAR VIEWS (All Schemas) ==='
\echo ''
SELECT 
    schemaname,
    viewname as view_name,
    pg_get_viewdef(viewname::regclass, true) as definition
FROM pg_views
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY schemaname, viewname;

\echo ''
\echo '=== ALL MATERIALIZED VIEWS (All Schemas) ==='
\echo ''
SELECT 
    schemaname,
    matviewname as view_name,
    pg_get_viewdef(matviewname::regclass, true) as definition
FROM pg_matviews
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY schemaname, matviewname;

\echo ''
\echo '=== TIMESCALEDB CONTINUOUS AGGREGATES ==='
\echo ''
-- Check if timescaledb extension exists first
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        RAISE NOTICE 'TimescaleDB extension found';
    ELSE
        RAISE NOTICE 'TimescaleDB extension NOT found';
    END IF;
END $$;

SELECT 
    view_name,
    view_definition,
    materialized_only,
    finalized
FROM timescaledb_information.continuous_aggregates
ORDER BY view_name;

\echo ''
\echo '=== CONTINUOUS AGGREGATE POLICIES ==='
\echo ''
SELECT 
    view_name,
    policy_name,
    proc_name,
    start_offset,
    end_offset,
    schedule_interval,
    initial_start,
    timezone
FROM timescaledb_information.jobs
WHERE proc_name LIKE '%continuous_aggregate%'
ORDER BY view_name, policy_name;

\echo ''
\echo '=== ALL OBJECTS WITH "summary" OR "hourly" OR "daily" IN NAME ==='
\echo ''
SELECT 
    n.nspname as schema_name,
    c.relname as object_name,
    CASE c.relkind
        WHEN 'r' THEN 'table'
        WHEN 'v' THEN 'view'
        WHEN 'm' THEN 'materialized view'
        WHEN 'i' THEN 'index'
        WHEN 'S' THEN 'sequence'
        WHEN 's' THEN 'special'
        ELSE 'other'
    END as object_type
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
  AND (
    c.relname ILIKE '%summary%' 
    OR c.relname ILIKE '%hourly%' 
    OR c.relname ILIKE '%daily%'
    OR c.relname ILIKE '%weekly%'
    OR c.relname ILIKE '%monthly%'
    OR c.relname ILIKE '%aggregate%'
  )
ORDER BY n.nspname, c.relname;

\echo ''
\echo '=== INDEXES ON MATERIALIZED VIEWS ==='
\echo ''
SELECT 
    schemaname,
    tablename as view_name,
    indexname,
    indexdef
FROM pg_indexes
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
  AND tablename IN (
    SELECT matviewname 
    FROM pg_matviews 
    WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
  )
ORDER BY schemaname, tablename, indexname;
EOF

echo "Querying database for aggregate views..."
echo ""

# Execute the query and capture output
OUTPUT=$(kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -f - < "$SQL_FILE" 2>&1)

# Clean up
rm "$SQL_FILE"

# Display output
echo "$OUTPUT"

# Count views found
VIEW_COUNT=$(echo "$OUTPUT" | grep -E "(view_name|matviewname|viewname|row)" | grep -v "NOTICE" | wc -l | tr -d ' ')

echo ""
echo -e "${BLUE}Found $VIEW_COUNT view-related entries${NC}"
echo ""

# Also try direct queries for known view names
echo -e "${YELLOW}=== Checking for Known Views ===${NC}"
KNOWN_VIEWS=("energy_hourly_summary" "energy_daily_summary" "energy_monthly_summary" "energy_hourly" "heatpump_daily_summary")

for view in "${KNOWN_VIEWS[@]}"; do
  echo -n "Checking for $view... "
  EXISTS=$(kubectl exec timescaledb-0 -n heatpump-mqtt -- \
    env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -tAc \
    "SELECT EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE c.relname = '$view' AND n.nspname NOT IN ('pg_catalog', 'information_schema'));" 2>/dev/null || echo "false")
  
  if [ "$EXISTS" = "t" ]; then
    echo -e "${GREEN}✓ EXISTS${NC}"
  else
    echo -e "${RED}✗ NOT FOUND${NC}"
  fi
done

echo ""
echo -e "${YELLOW}=== All Objects in Database ===${NC}"
kubectl exec timescaledb-0 -n heatpump-mqtt -- \
  env PGPASSWORD="$DB_PASSWORD" psql -h localhost -U "$DB_USER" -d "$DB_NAME" -c "
SELECT 
    n.nspname as schema,
    c.relname as name,
    CASE c.relkind
        WHEN 'r' THEN 'table'
        WHEN 'v' THEN 'view'
        WHEN 'm' THEN 'materialized view'
        WHEN 'i' THEN 'index'
        ELSE 'other'
    END as type
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
  AND c.relkind IN ('v', 'm')
ORDER BY n.nspname, c.relname;
" 2>&1 | grep -v "NOTICE" || true

echo ""
echo -e "${GREEN}=== Extraction Complete ===${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Review the output above to identify views that need to be added to migrations"
echo "2. For each view, create a new migration file in migrations/ directory"
echo "3. Use the CREATE statements from the definitions above"
echo ""
echo -e "${YELLOW}To get the full CREATE statement for a specific view:${NC}"
echo "  ./get-view-definition.sh <view_name>"
