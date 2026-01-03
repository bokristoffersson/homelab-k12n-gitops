# Monitoring

## Overview

This document describes how to monitor the health and performance of the homelab streaming infrastructure.

## Key Metrics

### Kafka/Redpanda

#### Topic Metrics

```bash
# View topic details
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic describe homelab.energy
```

**Watch for:**
- Partition count
- Replication status
- Message count
- Disk usage

#### Consumer Lag

```bash
# Check all consumer groups
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group list

# Describe specific group
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe timescaledb-energy
```

**Healthy Lag:**
- Energy: < 100 messages
- Heatpump: < 50 messages
- Temperature: < 20 messages

#### Cluster Health

```bash
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk cluster health
```

### TimescaleDB

#### Database Size

```sql
SELECT
    pg_size_pretty(pg_database_size('homelab')) AS database_size;
```

#### Table Sizes

```sql
SELECT
    hypertable_name,
    pg_size_pretty(total_bytes) AS total_size,
    pg_size_pretty(index_bytes) AS index_size
FROM timescaledb_information.hypertables
JOIN timescaledb_information.dimensions USING (hypertable_schema, hypertable_name);
```

#### Recent Inserts

```sql
SELECT
    'energy_readings' AS table_name,
    COUNT(*) AS count,
    MAX(timestamp) AS latest
FROM energy_readings
WHERE timestamp > NOW() - INTERVAL '5 minutes'
UNION ALL
SELECT
    'heatpump_status',
    COUNT(*),
    MAX(timestamp)
FROM heatpump_status
WHERE timestamp > NOW() - INTERVAL '5 minutes';
```

#### Connection Count

```sql
SELECT count(*) FROM pg_stat_activity;
```

### Redpanda Connect

#### Metrics Endpoint

Redpanda Connect exposes metrics on port 4195:

```bash
kubectl port-forward -n timescaledb deployment/redpanda-connect 4195:4195
curl http://localhost:4195/metrics
```

**Key Metrics:**
- `benthos_input_received_total`
- `benthos_output_sent_total`
- `benthos_processor_error_total`
- `benthos_input_latency`

#### Logs

```bash
# TimescaleDB sink
kubectl logs -n timescaledb deployment/redpanda-connect -f

# Heatpump settings processor
kubectl logs -n heatpump-settings deployment/redpanda-connect -f
```

### MQTT Bridge

#### Bridge Status

```bash
kubectl get pods -n mqtt-kafka-bridge
kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge -f
```

**Look for:**
- MQTT connection status
- Kafka producer status
- Message throughput
- Errors and retries

### Application Services

#### Energy WebSocket

```bash
kubectl get pods -n energy-ws
kubectl logs -n energy-ws deployment/energy-ws -f
```

**Metrics:**
- Connected clients
- Messages sent per second
- WebSocket errors

#### Homelab API

```bash
kubectl get pods -n homelab-api
kubectl logs -n homelab-api deployment/homelab-api -f
```

**Endpoints:**
- `/health` - Health check
- `/metrics` - Prometheus metrics (if implemented)

## Alerting

### Critical Alerts

These require immediate attention:

| Alert | Condition | Action |
|-------|-----------|--------|
| Redpanda Down | All brokers unreachable | Check cluster, restart if needed |
| TimescaleDB Down | Database unreachable | Check deployment, restore from backup |
| High Consumer Lag | Lag > 1000 messages | Scale consumers, check for issues |
| Disk Full | Disk usage > 90% | Clean old data, increase PVC size |
| No Data Ingestion | No inserts for 5 minutes | Check MQTT bridge, IoT devices |

### Warning Alerts

These indicate potential issues:

| Alert | Condition | Action |
|-------|-----------|--------|
| Moderate Lag | Lag > 100 messages | Monitor, may need scaling |
| Slow Queries | Query time > 5s | Optimize queries, check indexes |
| High Memory | Memory > 80% | Consider scaling up |
| Backup Failure | Backup job failed | Investigate and retry |

## Dashboards

### Backstage

View all components and their health:

1. Navigate to Backstage catalog
2. View Kubernetes tab for pod status
3. View Kafka tab for consumer group lag
4. View Docs tab for component documentation

### Grafana

Create dashboards for:

**System Overview:**
- Message throughput
- Consumer lag
- Database write rate
- Error rate

**Energy Dashboard:**
- Real-time power consumption
- Hourly/daily totals
- Cost estimation
- Trends

**Heatpump Dashboard:**
- Current temperature vs target
- Operating mode
- Power consumption
- Runtime statistics

## Troubleshooting

### No Data in TimescaleDB

1. **Check data source**:
   ```bash
   mosquitto_sub -h mosquitto.mosquitto.svc -t "homelab/#" -C 5
   ```

2. **Check MQTT bridge**:
   ```bash
   kubectl logs -n mqtt-kafka-bridge deployment/mqtt-kafka-bridge | tail -20
   ```

3. **Check Kafka**:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic consume homelab.energy --num 5
   ```

4. **Check Redpanda Connect**:
   ```bash
   kubectl logs -n timescaledb deployment/redpanda-connect | tail -20
   ```

5. **Check TimescaleDB**:
   ```sql
   SELECT COUNT(*) FROM energy_readings WHERE timestamp > NOW() - INTERVAL '1 minute';
   ```

### High Lag

1. **Identify bottleneck**:
   ```bash
   kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group describe timescaledb-energy
   ```

2. **Check consumer resources**:
   ```bash
   kubectl top pod -n timescaledb
   ```

3. **Scale if needed**:
   ```bash
   kubectl scale -n timescaledb deployment/redpanda-connect --replicas=2
   ```

4. **Check database performance**:
   ```sql
   SELECT * FROM pg_stat_activity WHERE state = 'active';
   ```

### Pods Crashing

1. **Get pod status**:
   ```bash
   kubectl get pods -A | grep -v Running
   ```

2. **View logs**:
   ```bash
   kubectl logs -n <namespace> <pod-name> --previous
   ```

3. **Describe pod**:
   ```bash
   kubectl describe pod -n <namespace> <pod-name>
   ```

4. **Check events**:
   ```bash
   kubectl get events -n <namespace> --sort-by='.lastTimestamp'
   ```

## Maintenance Tasks

### Daily

- [ ] Check consumer group lag
- [ ] Verify recent data in TimescaleDB
- [ ] Check backup job status

### Weekly

- [ ] Review error logs
- [ ] Check disk usage trends
- [ ] Verify dashboard accuracy

### Monthly

- [ ] Review retention policies
- [ ] Clean up old consumer groups
- [ ] Update documentation
- [ ] Review performance trends

## Useful Commands

### Quick Health Check

```bash
#!/bin/bash
echo "=== Redpanda Health ==="
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk cluster health

echo "=== Consumer Groups ==="
kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk group list

echo "=== TimescaleDB Status ==="
kubectl get pods -n timescaledb

echo "=== Recent Data ==="
kubectl exec -n timescaledb deployment/timescaledb -- \
  psql -U postgres -c "SELECT COUNT(*) FROM energy_readings WHERE timestamp > NOW() - INTERVAL '5 minutes';"
```

Save as `health-check.sh` and run periodically.
