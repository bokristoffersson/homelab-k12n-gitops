# Adding New Attributes to Database

This guide explains how to add a new attribute/column to the database when it appears in Redpanda messages.

## Overview

When a new attribute appears in Redpanda messages and needs to be stored in the database, you need to:

1. Add the field/tag mapping to the ConfigMap
2. Create a migration script to add the column
3. Commit and deploy via GitOps

## Step-by-Step Process

### Step 1: Add Field/Tag Mapping to ConfigMap

Edit `configmap.yaml` and add the new field/tag to the appropriate pipeline's `tags:` or `fields:` section.

**Example:** Adding a new `humidity_pct` field to the telemetry pipeline:

```yaml
pipelines:
  - name: "telemetry"
    topic: "telemetry"
    table: "telemetry"
    data_type: "timeseries"
    fields:
      flow_temp_c:
        path: "$.flow_temp"
        type: "float"
      humidity_pct:          # NEW
        path: "$.humidity"  # NEW
        type: "float"       # NEW
```

**Field Types:**
- `float` - For decimal numbers (maps to `DOUBLE PRECISION` in PostgreSQL)
- `int` - For integers (maps to `BIGINT` in PostgreSQL)
- `text` - For strings (maps to `TEXT` in PostgreSQL)
- `bool` - For booleans (maps to `BOOLEAN` in PostgreSQL)

**Tags vs Fields:**
- **Tags**: Used for filtering/indexing (stored as `TEXT` columns)
- **Fields**: Used for metrics/measurements (stored with appropriate types)

### Step 2: Create Migration Script

Create a new migration file in `migrations/` directory:

- Name it sequentially: `003_add_humidity_pct.sql` (use the next number)
- Use `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` for idempotency
- Match the column name and type from the config

**Example for timeseries table:**

```sql
-- Add humidity_pct column to telemetry table
ALTER TABLE telemetry 
ADD COLUMN IF NOT EXISTS humidity_pct DOUBLE PRECISION;
```

**Example for static table:**

```sql
-- Add firmware_version to devices table
ALTER TABLE devices 
ADD COLUMN IF NOT EXISTS firmware_version TEXT;
```

**PostgreSQL Type Mapping:**
- Config `float` → SQL `DOUBLE PRECISION`
- Config `int` → SQL `BIGINT`
- Config `text` → SQL `TEXT`
- Config `bool` → SQL `BOOLEAN`

### Step 3: Update Kustomization

Add the new migration file to `kustomization.yaml` in the `configMapGenerator` section:

```yaml
configMapGenerator:
  - name: redpanda-sink-migrations
    files:
      - migrations/001_init_timeseries_tables.sql
      - migrations/002_init_static_tables.sql
      - migrations/003_add_humidity_pct.sql  # NEW
```

### Step 4: Commit and Deploy

1. Commit all changes (configmap.yaml, migration script, kustomization.yaml) to Git
2. Push to the repository
3. Flux will automatically:
   - Detect the new migration script
   - Run the migration job (which executes all migration scripts in order)
   - Update the ConfigMap
   - Restart the deployment with the new config

### Step 5: Verify

**Check migration job logs:**
```bash
kubectl logs -n redpanda-sink job/redpanda-sink-migration
```

**Verify column exists in database:**
```bash
# Connect to database and check table schema
psql $DATABASE_URL -c "\d telemetry"
```

**Check application logs:**
```bash
kubectl logs -n redpanda-sink deployment/redpanda-sink
```

New data should flow without errors once the column exists.

## Complete Example

**Scenario:** Add `voltage_v` field from Redpanda message `{"device_id": "hp-01", "voltage": 230.5}`

### 1. Update `configmap.yaml`:

```yaml
pipelines:
  - name: "telemetry"
    fields:
      power_w:
        path: "$.power"
        type: "int"
      voltage_v:           # NEW
        path: "$.voltage"   # NEW
        type: "float"      # NEW
```

### 2. Create `migrations/003_add_voltage_v.sql`:

```sql
-- Add voltage_v column to telemetry table
ALTER TABLE telemetry 
ADD COLUMN IF NOT EXISTS voltage_v DOUBLE PRECISION;
```

### 3. Update `kustomization.yaml`:

```yaml
configMapGenerator:
  - name: redpanda-sink-migrations
    files:
      - migrations/001_init_timeseries_tables.sql
      - migrations/002_init_static_tables.sql
      - migrations/003_add_voltage_v.sql  # NEW
```

### 4. Commit & Push

```bash
git add configmap.yaml migrations/003_add_voltage_v.sql kustomization.yaml
git commit -m "Add voltage_v field to telemetry pipeline"
git push
```

GitOps handles the rest automatically!

## Important Notes

- **Column names must match** between configmap (field/tag name) and migration (column name)
- **Column types must be compatible** (see mapping above)
- **Migrations are idempotent** - safe to re-run if needed
- **The application dynamically builds queries** - no code changes needed once column exists
- **Migration order matters** - files are executed in alphabetical/numerical order
- **Always test migrations** in a development environment first

## Troubleshooting

**Migration job fails:**
- Check job logs: `kubectl logs -n redpanda-sink job/redpanda-sink-migration`
- Verify database connection: Check `timescaledb-secret` exists
- Verify migration syntax: Test SQL manually against database

**Application fails after migration:**
- Check application logs for column errors
- Verify column exists: `\d table_name` in psql
- Verify configmap field name matches column name exactly

**Column not appearing in data:**
- Verify JSONPath in configmap matches actual message structure
- Check application logs for JSONPath extraction errors
- Verify field type matches the actual data type in messages

