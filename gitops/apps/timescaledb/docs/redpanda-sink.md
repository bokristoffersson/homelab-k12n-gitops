# Redpanda Sink

The sink is a Redpanda Connect Deployment (`redpanda-connect` in the `timescaledb`
namespace) that reads telemetry topics and inserts rows into TimescaleDB. It replaced
an earlier custom service called redpanda-sink; there is no application code to build.

```
Redpanda topic ──► mapping ──► INSERT into table
```

## Pipelines

One pipeline per file in `gitops/apps/timescaledb/configs/`:

| File | Topic | Consumer group | Table |
|---|---|---|---|
| `energy.yaml` | `homelab-energy-realtime` | `timescaledb-energy` | `energy_consumption` |
| `heatpump.yaml` | `homelab-heatpump-telemetry` | `timescaledb-heatpump` | `heatpump_status` |
| `temperature.yaml` | `homelab-temperature-indoor`, `homelab-temperature-outdoor` | `timescaledb-temperature` | `temperature_sensors` |

Each pipeline has three parts: a `kafka` input, a `mapping` processor that turns the
message's `ts`, `tags` and `fields` into table columns, and a `sql_insert` output.
Database credentials come from the `timescaledb-credentials` secret.

## Changing a pipeline

1. Edit the file under `configs/`.
2. If the change needs a new column, add a migration first. See [Migrations](migrations.md).
3. Merge the PR, then restart the sink so it loads the new ConfigMap:

   ```bash
   kubectl rollout restart deployment/redpanda-connect -n timescaledb
   ```

## Checking it

```bash
# Is it running, and what is it logging?
kubectl get pods -n timescaledb
kubectl logs -n timescaledb deployment/redpanda-connect --tail=50

# Is it keeping up?
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe timescaledb-energy
```

The `monitoring-health-check` CronJob alerts in Slack when a table stops receiving rows
or a consumer group falls more than 1000 messages behind.

## Troubleshooting

| Symptom | Check |
|---|---|
| Lag grows | Sink logs for insert errors; `kubectl top pod -n timescaledb` for a starved database |
| Rows stop arriving but lag is zero | Nothing is being produced: check `mqtt-kafka-bridge` and the device |
| Insert errors after a mapping change | A column name or type does not match the table |
| Sink restarts in a loop | Database unreachable or wrong credentials |

Messages older than the topic's retention (1 to 24 hours) are gone, so a sink that is
down longer than that leaves a gap.
