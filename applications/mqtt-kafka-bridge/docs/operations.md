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

### Health Checks

The bridge exposes health endpoints:

- `/health`: Overall health status
- `/metrics`: Prometheus metrics

### Key Metrics

- `mqtt_messages_received_total`: Total MQTT messages received
- `kafka_messages_published_total`: Total Kafka messages published
- `bridge_errors_total`: Total errors encountered

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

### High Lag or Message Loss

1. Check consumer group lag:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe mqtt-bridge
   ```

2. Scale up if needed:
   ```bash
   kubectl scale -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge --replicas=2
   ```

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
