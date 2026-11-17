# Redpanda Connect Sink

## Overview

The Redpanda Connect sink is responsible for consuming messages from Kafka topics and inserting them into TimescaleDB tables.

## Architecture

```
Kafka Topics → Redpanda Connect → TimescaleDB
```

## Consumer Groups

The sink operates three consumer groups:

### timescaledb-energy

- **Topic**: `homelab.energy`
- **Table**: `energy_readings`
- **Throughput**: ~1-10 msgs/sec
- **Lag Target**: < 100 messages

### timescaledb-heatpump

- **Topic**: `homelab.heatpump`
- **Table**: `heatpump_status`
- **Throughput**: ~0.5 msgs/sec
- **Lag Target**: < 50 messages

### timescaledb-temperature

- **Topic**: `homelab.temperature`
- **Table**: `temperature_readings`
- **Throughput**: ~0.2 msgs/sec (per sensor)
- **Lag Target**: < 20 messages

## Configuration

Redpanda Connect is configured via YAML pipelines:

```yaml
input:
  kafka:
    addresses: ["redpanda-v2.redpanda-v2.svc.cluster.local:9092"]
    topics: ["homelab.energy"]
    consumer_group: timescaledb-energy

pipeline:
  processors:
    - mapping: |
        root.timestamp = this.timestamp
        root.device_id = this.device_id
        root.total_energy_kwh = this.total_energy_kwh
        root.power_kw = this.power_kw

output:
  sql_insert:
    driver: postgres
    dsn: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@timescaledb:5432/${POSTGRES_DB}
    table: energy_readings
    columns: [timestamp, device_id, total_energy_kwh, power_kw]
```

## Monitoring

### View Consumer Group Lag

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
  rpk group describe timescaledb-energy
```

### Check Sink Logs

```bash
kubectl logs -n timescaledb deployment/redpanda-connect -f
```

### Metrics

The sink exposes Prometheus metrics on port 4195:

- `benthos_input_received_total`: Messages received from Kafka
- `benthos_output_sent_total`: Messages written to TimescaleDB
- `benthos_processor_error_total`: Processing errors

## Troubleshooting

### High Lag

If consumer group lag is high:

1. Check database write performance:
   ```sql
   SELECT * FROM pg_stat_activity WHERE state = 'active';
   ```

2. Increase parallelism in Redpanda Connect config

3. Check for database locks or slow queries

### Messages Not Appearing

1. Verify Kafka topic has messages:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- \
     rpk topic consume homelab.energy --num 10
   ```

2. Check Redpanda Connect logs for errors

3. Verify database credentials and connectivity

## Scaling

To handle increased load:

```bash
kubectl scale -n timescaledb deployment/redpanda-connect --replicas=2
```

**Note**: Ensure consumer group partitioning allows multiple consumers.
