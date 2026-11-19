# Data Flow

How a reading gets from a device to a chart, and how a command gets back to a device.

## Telemetry: device to dashboard

```
Device ─► Mosquitto ─► mqtt-kafka-bridge ─► Redpanda ─┬─► Redpanda Connect sink ─► TimescaleDB ─► homelab-api ─► heatpump-web
 (MQTT)                (Redpanda Connect)             │
                                                      ├─► energy-ws ─► WebSocket clients
                                                      └─► homelab-settings (reported device state)
```

| Step | Component | What happens |
|---|---|---|
| 1 | Device | Publishes to Mosquitto (`mosquitto.mosquitto.svc:1883`; exposed on the LAN at port 1883) |
| 2 | `mqtt-kafka-bridge` | Subscribes to the device topics, reshapes each payload and produces to a Redpanda topic |
| 3 | Redpanda | Holds the message for its retention period so each consumer reads at its own pace |
| 4 | Redpanda Connect sink (ns `timescaledb`) | Inserts rows into TimescaleDB |
| 5 | `homelab-api` | Serves the stored data over REST. It only reads. |
| 6 | `heatpump-web` | Fetches from the API and renders charts |

### Sources

| Device | MQTT topic | Redpanda topic | TimescaleDB table |
|---|---|---|---|
| SaveEye energy meter | `saveeye/telemetry` | `homelab-energy-realtime` | `energy_consumption` |
| ThermIQ heat pump module | `thermiq_heatpump/data` | `homelab-heatpump-telemetry` | `heatpump_status` |
| Shelly H&T Gen3 | `shellyhtg3-e4b32322a0f4/events/rpc` | `homelab-temperature-indoor` | `temperature_sensors` |
| Tasmota plugs | `tele/+/STATE`, `stat/+/POWER` | `homelab-plug-telemetry` | (state kept in homelab-settings) |

See [kafka-topics.md](kafka-topics.md) for retention and consumer groups.

Spot prices do not come through this pipeline: `spotprice-api` fetches them from Nord
Pool once a day and writes `spot_prices` itself.

### Storage

Raw telemetry tables are hypertables with a 90-day retention policy (`spot_prices`: 30
days). Continuous aggregates such as `energy_hourly` and `energy_daily_summary` are what
the API serves for longer ranges. The schema lives in `gitops/apps/timescaledb/migrations/`.

### Real-time path

`energy-ws` consumes `homelab-energy-realtime` directly and pushes each reading to
connected WebSocket clients at `wss://homelab.k12n.com/ws/energy`, so the live power
gauge does not wait for the database.

## Commands: dashboard to device

```
heatpump-web ─► homelab-settings-api ─► PostgreSQL (setting + outbox row, one transaction)
                                              │
                               outbox processor ─► Mosquitto ─► device
                                              ▲                    │
                                              └── confirmation ◄───┘  (telemetry echo via Redpanda)
```

1. The API writes the new setting and an outbox row in one transaction.
2. The outbox processor publishes the command to MQTT.
3. A plug command counts as confirmed only when the plug echoes the matching state on
   its telemetry topic. Unconfirmed commands are republished, then marked failed.
4. A reconciler re-issues commands while a plug's desired and reported state disagree.

HomeKit commands from Homebridge skip steps 1 and 2 and go straight to Mosquitto. The
state echo keeps the database consistent. The full rules are in "Write Paths to
Devices" in [CLAUDE.md](../CLAUDE.md), and the design is recorded in
[ADR-001](adr/001-transactional-outbox-for-heatpump-settings.md).

## What happens when a part is down

| Component down | Effect |
|---|---|
| A consumer (sink, energy-ws) | Messages wait in Redpanda. The consumer resumes from its committed offset, as long as it returns within the topic's retention. |
| Redpanda | The bridge cannot produce; readings published meanwhile are lost. |
| TimescaleDB | The sink stops and catches up afterwards, within retention. |
| Mosquitto | Devices cannot publish, and commands stay queued in the outbox. |

Redpanda runs as a single node with one replica per topic, so it is not highly
available.

## Following a reading through the pipeline

```bash
# 1. Is the device publishing? (run `make port-mosquitto` first; the broker requires
#    a username and password)
mosquitto_sub -h localhost -u <user> -P <password> -t 'saveeye/telemetry' -C 1

# 2. Did it reach Redpanda?
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk topic consume homelab-energy-realtime --num 1

# 3. Is the sink keeping up?
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe timescaledb-energy

# 4. Are the bridge and sink healthy?
kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge --tail=20
kubectl logs -n timescaledb deployment/redpanda-connect --tail=20
```

The `monitoring-health-check` CronJob runs these checks every 15 minutes and posts to
Slack when data goes stale. See [monitoring.md](monitoring.md).
