# Database Migrations

This directory contains database migration scripts for the homelab-settings application.

## Migration Naming Convention

Migrations should be named using the following pattern:
```
{version}_{description}.sql
```

Example:
```
001_initial_schema.sql
002_add_index_on_device_id.sql
003_add_monitoring_columns.sql
```

## Creating a New Migration

1. Create a new SQL file with the next sequential version number
2. Include a comment at the top describing the migration
3. Write your DDL statements (ALTER TABLE, CREATE INDEX, etc.)
4. Add the migration to the kustomization.yaml configMapGenerator
5. Record the migration in the schema_migrations table

Example migration file (`002_add_index.sql`):
```sql
-- Migration: 002_add_index
-- Description: Add index on mode column for faster filtering

\c homelab_settings

-- Add your migration SQL here
CREATE INDEX IF NOT EXISTS idx_settings_mode ON settings (mode);

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (2, 'add_index')
ON CONFLICT (version) DO NOTHING;
```

## Updating Kustomization

After creating a new migration file, add it to `kustomization.yaml`:

```yaml
configMapGenerator:
  - name: postgres-migrations
    files:
      - migrations/001_initial_schema.sql
      - migrations/002_add_index.sql  # Add your new migration here
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
2. PostgreSQL is ready and accessible
3. New migration files are added to the ConfigMap

The migration runner script processes all `.sql` files in order and records them in the `schema_migrations` table.
