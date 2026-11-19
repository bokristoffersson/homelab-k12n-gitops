# Kafka Topics

Telemetry flows through a single-node Redpanda cluster in the `redpanda-v2` namespace.
Topics are created by a Job that runs `rpk`
(`gitops/apps/redpanda-v2/topics/topic-creator-job.yaml`); there is no operator.

## Topics

Every topic has one partition and one replica.

| Topic | Retention | MQTT source | Consumers |
|---|---|---|---|
| `homelab-energy-realtime` | 24 h | `saveeye/telemetry` | `timescaledb-energy`, `energy-ws` |
| `homelab-heatpump-telemetry` | 1 h | `thermiq_heatpump/data` | `timescaledb-heatpump`, `homelab-settings`, `homelab-settings-api` |
| `homelab-plug-telemetry` | 24 h | `tele/+/STATE`, `stat/+/POWER` | `homelab-settings-api` |
| `homelab-temperature-indoor` | 24 h | `shellyhtg3-e4b32322a0f4/events/rpc` | `timescaledb-temperature` |
| `homelab-temperature-outdoor` | 24 h | none yet | `timescaledb-temperature` |

Retention is short on purpose. TimescaleDB is the store of record; the topics only need
to cover a consumer being down for a while.

## Producers

`mqtt-kafka-bridge` (Redpanda Connect) is the only producer. It has one pipeline per
source in `gitops/apps/mqtt-kafka-bridge/configs/`, and each pipeline maps the device
payload to the message format its topic carries. Read those files for the exact fields.

The bridge only matches single-channel Tasmota topics. A plug with several channels
(`POWER1`, `POWER2`) needs new topic filters.

## Consumer groups

| Group | Service | Namespace | Purpose |
|---|---|---|---|
| `timescaledb-energy` | Redpanda Connect sink | `timescaledb` | Write energy readings to TimescaleDB |
| `timescaledb-heatpump` | Redpanda Connect sink | `timescaledb` | Write heat pump telemetry |
| `timescaledb-temperature` | Redpanda Connect sink | `timescaledb` | Write temperature readings |
| `homelab-settings` | Redpanda Connect | `homelab-settings` | Keep the reported heat pump settings current |
| `homelab-settings-api` | homelab-settings-api | `homelab-settings` | Confirm plug commands and adopt state changes made outside the API |
| `energy-ws` | energy-ws | `energy-ws` | Stream energy readings to WebSocket clients |

## Working with topics

```bash
# List topics and groups
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic list
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group list

# Inspect a topic and read a few messages
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic describe homelab-energy-realtime
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk topic consume homelab-energy-realtime --num 5

# Check consumer lag
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe timescaledb-energy
```

`make redpanda-list` and `make redpanda-consume TOPIC=<name>` wrap the first two.

## Adding a topic

1. Add a `create_topic <name> <retention ms>` line to
   `gitops/apps/redpanda-v2/topics/topic-creator-job.yaml`.
2. Add a pipeline under `gitops/apps/mqtt-kafka-bridge/configs/` that produces to it,
   and list the file in that app's `kustomization.yaml`.
3. Add or extend a consumer.
4. Name it `homelab-<domain>-<kind>`, in lowercase with hyphens.

Do not create topics by hand with `rpk topic create`; the Job is what recreates them
on a rebuilt cluster.
