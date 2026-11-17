# Database Migrations

This directory contains database migration scripts for the timescaledb application.

## Migration Naming Convention

Migrations should be named using the following pattern:
```
{version}_{description}.sql
```

Example:
```
001_initial_schema.sql
002_add_aggregation_view.sql
003_modify_retention_policy.sql
```

## Creating a New Migration

1. Create a new SQL file with the next sequential version number
2. Include a comment at the top describing the migration
3. Write your DDL statements (ALTER TABLE, CREATE INDEX, etc.)
4. Add the migration to the kustomization.yaml configMapGenerator
5. Record the migration in the schema_migrations table

Example migration file (`002_add_continuous_aggregate.sql`):
```sql
-- Migration: 002_add_continuous_aggregate
-- Description: Add continuous aggregate for hourly energy consumption

\c telemetry

-- Add your migration SQL here
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_consumption_hourly
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 hour', time) AS bucket,
  device_serial,
  AVG(active_power_total) AS avg_power,
  MAX(active_energy_total) AS max_energy
FROM energy_consumption
GROUP BY bucket, device_serial;

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (2, 'add_continuous_aggregate')
ON CONFLICT (version) DO NOTHING;
```

## Updating Kustomization

After creating a new migration file, add it to `kustomization.yaml`:

```yaml
configMapGenerator:
  - name: timescaledb-migrations
    files:
      - migrations/001_initial_schema.sql
      - migrations/002_add_continuous_aggregate.sql  # Add your new migration here
      - migrations/run_migrations.sh
```

## Migration Tracking

The `schema_migrations` table tracks which migrations have been applied:
- `version`: Unique migration version number
- `name`: Description of the migration
- `applied_at`: Timestamp when the migration was applied

## Running Migrations

Migrations run automatically when:
1. The migration Job is deployed/updated
2. TimescaleDB is ready and accessible
3. New migration files are added to the ConfigMap

The migration runner script processes all `.sql` files in order and records them in the `schema_migrations` table.

## TimescaleDB-Specific Features

When working with TimescaleDB migrations, you can:
- Create/modify hypertables
- Add continuous aggregates
- Modify retention policies
- Create compression policies
- Add TimescaleDB-specific indexes
