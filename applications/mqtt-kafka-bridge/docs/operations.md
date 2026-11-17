# Operations

## Deployment

The service is deployed as a Kubernetes Deployment in the `mqtt-kafka-bridge` namespace.

```bash
kubectl get pods -n mqtt-kafka-bridge
```

## Monitoring

### Logs

View bridge logs:

```bash
kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge -f
```

### Health

Redpanda Connect serves `/ping` and `/ready` on port 4195; the Deployment's probes use
them. Prometheus-format metrics are at `/metrics` on the same port:

```bash
kubectl port-forward -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge 4195:4195
curl -s localhost:4195/metrics | grep -E "input_received|output_sent|error"
```

The `monitoring-health-check` CronJob alerts in Slack when the bridge logs lost MQTT
connections.

## Troubleshooting

### Bridge Not Receiving MQTT Messages

1. Check Mosquitto is running:
   ```bash
   kubectl get pods -n mosquitto
   ```

2. Verify MQTT topic subscriptions:
   ```bash
   kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge | grep "Subscribed"
   ```

### Bridge Not Publishing to Kafka

1. Check Redpanda cluster health:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk cluster health
   ```

2. Verify Kafka connection:
   ```bash
   kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge | grep "Kafka"
   ```

### Messages missing downstream

The bridge is a producer, so it has no consumer group and no lag of its own.

1. Confirm the device is publishing (see the first section).
2. Confirm messages reach the topic:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
     rpk topic consume homelab-energy-realtime --num 1
   ```
3. If they do, the problem is in the consumer. Check its lag instead.

Run one replica only. Two replicas would each forward every MQTT message, and every
reading would be stored twice.

## Maintenance

### Restart

```bash
kubectl rollout restart -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge
```

### Update Configuration

Configuration is managed via ConfigMap. After updating:

```bash
kubectl rollout restart -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge
```
