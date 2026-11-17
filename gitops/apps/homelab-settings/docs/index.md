# Homelab Settings

The service that changes things: heat pump settings, smart plugs and plug schedules.
It lives in the `homelab-settings` namespace.

```
heatpump-web ──► homelab-settings-api ──► PostgreSQL (setting + outbox row)
                                               │
                              outbox processor ──► Mosquitto ──► device
                                                                   │
            homelab-settings-api ◄── Redpanda ◄── mqtt-kafka-bridge ◄┘  (state echo)
```

## Workloads

| Workload | Role |
|---|---|
| `homelab-settings-api` | REST API for settings, plugs and schedules. Also consumes device telemetry to confirm commands, runs the plug scheduler and the reconciler. |
| `homelab-settings-outbox-processor` | Publishes queued commands to MQTT and tracks delivery |
| `redpanda-connect` | Keeps the reported heat pump settings current. See [Redpanda Connect](redpanda-connect.md). |
| `postgres` | The `homelab_settings` database, on a 10 Gi Longhorn volume |
| `postgres-migration` Job | Applies the SQL files in `migrations/` |
| `postgres-backup` CronJob | Daily dump to S3 at 02:15 UTC |

Source code: `applications/homelab-settings-api` and
`applications/homelab-settings-outbox-processor`.

## How a command is delivered

1. The API writes the change and an `outbox` row in one transaction, so a command is
   never lost between "saved" and "sent".
2. The outbox processor publishes it to MQTT: `cmnd/{plug_id}/POWER` for plugs,
   `thermiq_heatpump/write` for the heat pump.
3. A plug command is confirmed only when the plug echoes the matching state. Until
   then it is republished after a timeout, up to a retry limit, then marked `failed`.
   A newer command for the same plug supersedes older undelivered ones.
4. Commands are level-triggered (`ON`, `OFF`), so sending one twice is harmless.

Failed commands raise a Slack alert through the `monitoring-health-check` CronJob.

## Desired and reported state

Each plug has a `desired_status` (what was asked for) and a `status` (what the plug last
reported). While they differ, the reconciler re-issues the command.

State changes made elsewhere, such as a HomeKit command through Homebridge or the
button on the plug, arrive as telemetry and are adopted as the new desired state, so
the reconciler does not fight them.

## Schedules

Plug schedules run in `Europe/Stockholm` time. Each schedule records when it last
fired, so a fire missed during a restart is caught up instead of skipped.

## Database

| Table | Contents |
|---|---|
| `settings` | Current heat pump settings per device |
| `outbox` | Queued, sent, confirmed and failed commands |
| `power_plugs` | Plugs with desired and reported state |
| `power_plug_schedules` | On/off schedules per plug |
| `schema_migrations` | Applied migrations |

Migrations follow the same workflow as TimescaleDB: add the file, list it in
`kustomization.yaml`, merge, then delete the `postgres-migration` Job so Flux reruns
it. `scripts/rerun-postgres-migration.sh` does the last step.

## Authorization

The API checks scopes on the Authelia access token: `read:settings`, `write:settings`,
`read:plugs`, `write:plugs`.
