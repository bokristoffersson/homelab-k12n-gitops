# Database Migrations

Schema changes are SQL files applied by a Kubernetes Job that Flux manages. They are
never run by hand with `kubectl exec`.

## How it works

- The files live in `gitops/apps/timescaledb/migrations/`, numbered `001_`, `002_`, …
- The app's `kustomization.yaml` packs them into the `timescaledb-migrations` ConfigMap.
- The `timescaledb-migration` Job mounts the ConfigMap and runs `run_migrations.sh`,
  which applies each file in order.
- Every migration records itself in `schema_migrations (version, name, applied_at)` and
  is written to be safe to run again.

## Adding a migration

1. Create the file, for example `migrations/008_add_something.sql`:

   ```sql
   -- Migration: 008_add_something
   -- Description: What this migration does

   \c telemetry

   ALTER TABLE temperature_sensors ADD COLUMN IF NOT EXISTS dew_point DOUBLE PRECISION;

   INSERT INTO schema_migrations (version, name)
   VALUES (8, 'add_something')
   ON CONFLICT (version) DO NOTHING;
   ```

2. List it in `gitops/apps/timescaledb/kustomization.yaml` under the
   `timescaledb-migrations` `configMapGenerator`. Without this step Flux never ships the
   file and the migration does not run.

3. Open a PR and merge it. Flux updates the ConfigMap.

4. Delete the Job so Flux recreates it with the new files:

   ```bash
   kubectl delete job -n timescaledb timescaledb-migration
   ```

5. Check the result:

   ```bash
   kubectl logs -n timescaledb job/timescaledb-migration
   ```

## Writing safe migrations

- Use `IF NOT EXISTS` / `IF EXISTS` so a rerun is harmless.
- End with the `schema_migrations` insert and `ON CONFLICT DO NOTHING`.
- Changing a continuous aggregate means dropping and recreating it; refresh it
  afterwards:

  ```sql
  CALL refresh_continuous_aggregate('energy_hourly', NULL, NULL);
  ```

- If the sink writes to the table, update its mapping in `configs/` in the same PR. See
  [Redpanda Sink](redpanda-sink.md).

## If a migration fails

1. Read the Job log to find the failing statement.
2. Fix the file in a new PR. Do not edit the database by hand.
3. Delete the Job again so it reruns.

For a migration that destroyed data, restore from the nightly backup
([BACKUP_SETUP.md](https://github.com/bokristoffersson/homelab-k12n-gitops/blob/main/docs/BACKUP_SETUP.md)).
