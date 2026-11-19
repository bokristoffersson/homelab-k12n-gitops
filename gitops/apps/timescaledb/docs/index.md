# TimescaleDB

The time-series database that stores the homelab's telemetry. It runs in the
`timescaledb` namespace as a single instance on a 50 Gi Longhorn volume; the database
is called `telemetry`.

```
Redpanda ──► Redpanda Connect sink ──► TimescaleDB ──► homelab-api, spotprice-api
```

## What runs here

| Workload | Purpose |
|---|---|
| `timescaledb` Deployment | The database |
| `redpanda-connect` Deployment | The sink: reads telemetry topics and inserts rows. See [Redpanda Sink](redpanda-sink.md). |
| `timescaledb-migration` Job | Applies the SQL files in `migrations/`. See [Migrations](migrations.md). |
| `timescaledb-backup` CronJob | Daily `pg_dump` to S3 at 02:00 UTC |

## Data

| Source topic | Table | Written by |
|---|---|---|
| `homelab-energy-realtime` | `energy_consumption` | sink |
| `homelab-heatpump-telemetry` | `heatpump_status` | sink |
| `homelab-temperature-indoor`, `homelab-temperature-outdoor` | `temperature_sensors` | sink |
| (Nord Pool API) | `spot_prices` | spotprice-api |

Raw tables keep 90 days (`spot_prices`: 30 days). Longer ranges come from continuous
aggregates. See [Schema](schema.md).

## Who reads it

- **homelab-api** serves energy, heat pump and temperature data. It only reads.
- **spotprice-api** reads and writes `spot_prices` and its device-token table.

## Access

```bash
kubectl exec -it -n timescaledb deployment/timescaledb -- psql -U postgres -d telemetry
```

Use this to look, not to change. Schema changes go through migrations.

## Backup and restore

See [docs/BACKUP_SETUP.md](https://github.com/bokristoffersson/homelab-k12n-gitops/blob/main/docs/BACKUP_SETUP.md).
