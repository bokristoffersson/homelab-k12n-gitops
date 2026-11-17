# Configuration

The bridge runs Redpanda Connect in `streams` mode: every YAML file in
`gitops/apps/mqtt-kafka-bridge/configs/` is one independent pipeline.

## Pipelines

| File | MQTT topic | Redpanda topic | Device |
|---|---|---|---|
| `energy.yaml` | `saveeye/telemetry` | `homelab-energy-realtime` | SaveEye energy meter |
| `heatpump.yaml` | `thermiq_heatpump/data` | `homelab-heatpump-telemetry` | ThermIQ heat pump module |
| `temperature-indoor.yaml` | `shellyhtg3-e4b32322a0f4/events/rpc` | `homelab-temperature-indoor` | Shelly H&T Gen3 |
| `power-plugs.yaml` | `tele/+/STATE` | `homelab-plug-telemetry` | Tasmota plugs, periodic state |
| `power-plugs-stat.yaml` | `stat/+/POWER` | `homelab-plug-telemetry` | Tasmota plugs, state transitions |

The plug pipelines match single-channel devices only. A plug that reports `POWER1` and
`POWER2` needs additional topic filters.

## Anatomy of a pipeline

```yaml
input:
  mqtt:
    urls: [tcp://mosquitto.mosquitto.svc.cluster.local:1883]
    topics: [saveeye/telemetry]
    user: ${MQTT_USER}
    password: ${MQTT_PASSWORD}
    qos: 1

pipeline:
  processors:
    - mapping: |
        root.ts = now().ts_format("2006-01-02T15:04:05.999999999Z07:00")
        root.tags.device_serial = this.saveeyeDeviceSerialNumber | ""
        root.fields.active_power_total = this.activeActualConsumption.total
        # ...

output:
  kafka:
    addresses: [${REDPANDA_BROKERS}]
    topic: homelab-energy-realtime
```

## Message format

The sensor pipelines (energy, heat pump, temperature) produce the same shape:

| Key | Content |
|---|---|
| `ts` | Timestamp set by the bridge when the message arrived |
| `tags` | Identity: device serial or ID, location, meter type |
| `fields` | The measurements |

The plug pipelines produce a flat message instead:

| Key | Content |
|---|---|
| `ts` | Timestamp set by the bridge |
| `plug_id` | Taken from the MQTT topic |
| `status` | `true` when the plug is on |
| `origin` | `stat` for a state transition, `tele` for the periodic report. homelab-settings-api treats the two differently. |

## Environment

| Variable | Source | Purpose |
|---|---|---|
| `MQTT_USER`, `MQTT_PASSWORD` | `mqtt-credentials` SealedSecret | Mosquitto login |
| `REDPANDA_BROKERS` | Deployment | Redpanda address |

## Adding a device

1. Add a pipeline file under `configs/` and list it in the app's `kustomization.yaml`.
2. If it needs a new topic, add it to
   `gitops/apps/redpanda-v2/topics/topic-creator-job.yaml`.
3. Add a consumer, for example a sink pipeline in `gitops/apps/timescaledb/configs/`.
4. After the merge, restart the bridge so it loads the new ConfigMap.
