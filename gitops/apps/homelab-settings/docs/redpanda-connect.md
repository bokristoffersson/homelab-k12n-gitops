# Redpanda Connect

A small Redpanda Connect Deployment in the `homelab-settings` namespace keeps the
`settings` table in step with what the heat pump actually reports.

```
homelab-heatpump-telemetry ──► mapping ──► UPSERT into settings
```

It does not send commands. Commands go out through the outbox processor.

## Pipeline

Defined in `gitops/apps/homelab-settings/configs/homelab-settings.yaml`.

| Part | What it does |
|---|---|
| Input | Consumes `homelab-heatpump-telemetry` with consumer group `homelab-settings` |
| Mapping | Picks the settings fields out of the telemetry: target temperature, mode, the heating curve values and heat stop |
| Output | `INSERT … ON CONFLICT (device_id) DO UPDATE` into `settings`, and only when a value differs, so an unchanged reading does not touch the row |

The heat pump reports its full state every cycle, which is how a setting changed on
the heat pump's own panel ends up in the database.

## Changing it

Edit the config file, merge the PR, then restart:

```bash
kubectl rollout restart deployment/redpanda-connect -n homelab-settings
```

If a new field needs a column, add a migration in the same PR.

## Checking it

```bash
kubectl logs -n homelab-settings deployment/redpanda-connect --tail=50
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe homelab-settings
```

| Symptom | Check |
|---|---|
| Settings in the UI do not follow the heat pump | Lag on the `homelab-settings` group, then the logs for SQL errors |
| SQL errors after a config change | A column in the query does not exist: the migration has not run |
| No messages at all | The heat pump module is not publishing: check `mqtt-kafka-bridge` |
