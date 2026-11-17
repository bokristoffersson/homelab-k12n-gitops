# MQTT-Kafka Bridge

The bridge moves device messages from Mosquitto into Redpanda. It is a
[Redpanda Connect](https://docs.redpanda.com/redpanda-connect/) Deployment in the
`mqtt-kafka-bridge` namespace; there is no application code, only pipeline
configuration.

```
Devices ──► Mosquitto ──► mqtt-kafka-bridge ──► Redpanda
```

Each pipeline subscribes to MQTT topics, reshapes the device's payload into a common
message format, and produces it to one Redpanda topic. Everything downstream (the
TimescaleDB sink, energy-ws, homelab-settings) reads from Redpanda.

- [Configuration](configuration.md): the pipelines and the message format
- [Operations](operations.md): checking, restarting and troubleshooting

## Related components

| Component | Relation |
|---|---|
| Mosquitto | Source. The bridge logs in with the `mqtt-credentials` secret. |
| Redpanda | Destination. Topics are created by the `rpk` Job in `redpanda-v2`. |
| TimescaleDB sink | Consumes the telemetry topics and writes them to the database |
| energy-ws | Consumes `homelab-energy-realtime` for live updates |
| homelab-settings | Consumes heat pump and plug telemetry to track reported state |
