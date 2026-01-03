# Database Migrations

## Overview

Database schema changes are managed through Kubernetes Jobs that run migration SQL scripts.

## Migration Structure

Migrations are located in `gitops/apps/base/timescaledb/migrations/`:

```
migrations/
├── README.md
├── 001_initial_schema.sql
├── 002_add_indexes.sql
└── 003_continuous_aggregates.sql
```

## Running Migrations

Migrations are executed automatically via Kubernetes Job:

```bash
kubectl get jobs -n timescaledb
kubectl logs -n timescaledb job/timescaledb-migration
```

## Creating a New Migration

1. **Create migration file**: `migrations/00X_description.sql`

2. **Write idempotent SQL**:
   ```sql
   -- migrations/004_add_humidity.sql

   -- Add column if it doesn't exist
   DO $$
   BEGIN
       IF NOT EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_name='temperature_readings'
           AND column_name='humidity'
       ) THEN
           ALTER TABLE temperature_readings
           ADD COLUMN humidity DOUBLE PRECISION;
       END IF;
   END $$;
   ```

3. **Update ConfigMap**: Add migration to the ConfigMap in `migration-job.yaml`

4. **Apply and run**:
   ```bash
   kubectl apply -f gitops/apps/base/timescaledb/migration-job.yaml
   kubectl create job --from=cronjob/timescaledb-migration migration-manual -n timescaledb
   ```

## Best Practices

### Always Use Idempotent Operations

- Use `IF NOT EXISTS` for CREATE statements
- Use `IF EXISTS` for DROP statements
- Check for existence before ALTER

### Test Migrations Locally

```bash
# Connect to TimescaleDB pod
kubectl exec -it -n timescaledb deployment/timescaledb -- psql -U postgres

# Test migration SQL
\i /path/to/migration.sql
```

### Backup Before Major Changes

```bash
# Trigger manual backup
kubectl create job --from=cronjob/timescaledb-backup backup-manual -n timescaledb
```

## Rollback

If a migration fails:

1. **Check logs**:
   ```bash
   kubectl logs -n timescaledb job/timescaledb-migration
   ```

2. **Restore from backup** if needed:
   ```bash
   # Download latest backup from S3
   # Restore to database
   ```

3. **Fix migration** and re-run

## Schema Versioning

Track applied migrations in a dedicated table:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    description TEXT,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

-- After each migration
INSERT INTO schema_migrations (version, description)
VALUES (4, 'Add humidity column');
```

## Continuous Aggregate Refresh

When modifying continuous aggregates:

```sql
-- Refresh after schema change
CALL refresh_continuous_aggregate('energy_hourly', NULL, NULL);
```
