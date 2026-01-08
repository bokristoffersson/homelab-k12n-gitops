# Grafana

Metrics visualization and monitoring platform for homelab telemetry data.

## Overview

Grafana provides rich, interactive dashboards for visualizing time-series data from TimescaleDB. It enables real-time monitoring of energy consumption, heat pump performance, and environmental sensors.

## Key Features

- **Time-series visualization**: Line charts, gauges, heatmaps
- **Real-time dashboards**: Auto-refreshing data views
- **TimescaleDB integration**: Native PostgreSQL data source
- **Alerting**: Threshold-based notifications
- **Custom dashboards**: JSON-based dashboard definitions
- **User management**: Role-based access control

## Architecture

```
┌──────────────────┐
│   TimescaleDB    │
│  (Data Source)   │
└────────┬─────────┘
         │ PostgreSQL protocol
         ▼
┌──────────────────┐
│     Grafana      │
│   - Dashboards   │
│   - Queries      │
│   - Visualizations
└────────┬─────────┘
         │ HTTPS
         ▼
┌──────────────────┐
│   Browser        │
│   (Users)        │
└──────────────────┘
```

## Deployment

- **Namespace**: `grafana`
- **URL**: `https://grafana.k12n.com`
- **Port**: 3000 (HTTP)
- **Replicas**: 1
- **Storage**: Persistent volume for dashboard configs
- **Resources**: 100m CPU, 256Mi memory

## Data Sources

### TimescaleDB

**Connection**:
- **Type**: PostgreSQL
- **Host**: `timescaledb-primary.timescaledb:5432`
- **Database**: `homelab`
- **User**: grafana_reader (read-only)
- **SSL**: Disabled (internal traffic)

**Tables**:
- `energy_data`: Power consumption metrics
- `heatpump_data`: Heat pump status
- `temperature_data`: Environmental sensors
- Continuous aggregates for 5m/1h/1d summaries

## Dashboards

### Energy Monitoring

**Panels**:
- Real-time power consumption (gauge)
- 24-hour power trend (line chart)
- Daily energy usage (bar chart)
- Cost estimation (single stat)
- Peak usage times (heatmap)

**Queries**:
```sql
-- Real-time power
SELECT
  time,
  power_w
FROM energy_data
WHERE time > NOW() - INTERVAL '1 hour'
ORDER BY time DESC;

-- 24-hour summary
SELECT
  time_bucket('5 minutes', time) AS bucket,
  AVG(power_w) as avg_power
FROM energy_data
WHERE time > NOW() - INTERVAL '24 hours'
GROUP BY bucket
ORDER BY bucket;
```

### Heat Pump Dashboard

**Panels**:
- Supply temperature (gauge)
- Return temperature (gauge)
- COP (Coefficient of Performance)
- Operating mode (stat)
- Temperature differential (line chart)
- Runtime hours (counter)

### Temperature Monitoring

**Panels**:
- Current temperature (gauge)
- Current humidity (gauge)
- 24-hour temperature trend
- 24-hour humidity trend
- Min/max/avg statistics

## Authentication

### Default Admin

**Username**: `admin`
**Password**: Configured via Secret

Change default password on first login:
1. Navigate to `https://grafana.k12n.com`
2. Log in with default credentials
3. Go to Profile → Change Password

### OAuth (Planned)

Integration with Authentik:
- **Provider**: Authentik OIDC
- **Auto-create users**: Enabled
- **Role mapping**: Based on Authentik groups

## Alerting

### Alert Rules

Example: High power consumption
```
Query: SELECT power_w FROM energy_data ORDER BY time DESC LIMIT 1
Condition: power_w > 5000
For: 5 minutes
Alert: Send notification
```

### Notification Channels

- Email
- Slack (planned)
- Webhooks (planned)

## Configuration

### Provisioning

Dashboards and data sources are provisioned via GitOps:

```
gitops/apps/base/grafana/
├── dashboards/
│   ├── energy.json
│   ├── heatpump.json
│   └── temperature.json
├── datasources/
│   └── timescaledb.yaml
└── grafana.ini
```

### Dashboard JSON

Dashboards are stored as JSON in Git:

```json
{
  "dashboard": {
    "title": "Energy Monitoring",
    "panels": [
      {
        "title": "Real-time Power",
        "type": "gauge",
        "targets": [
          {
            "rawSql": "SELECT power_w FROM energy_data..."
          }
        ]
      }
    ]
  }
}
```

## Storage

### Persistent Volume

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: grafana-data
  namespace: grafana
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 2Gi
```

Stores:
- User preferences
- Custom dashboards
- Alert history
- Sessions

## Performance

- **Query time**: <500ms for typical queries
- **Dashboard load**: <2s including all panels
- **Concurrent users**: Supports 50+ simultaneous users
- **Memory usage**: ~200MB RSS under normal load

## Monitoring

### Health Checks

```bash
# Check Grafana status
kubectl get pods -n grafana

# View logs
kubectl logs -n grafana -l app=grafana

# Test HTTP endpoint
curl https://grafana.k12n.com/api/health
```

### Metrics

Grafana exposes Prometheus metrics:

```
GET /metrics
```

Key metrics:
- `grafana_dashboard_get_duration_seconds`
- `grafana_database_queries_total`
- `grafana_api_response_status`

## Backup

### Dashboard Export

Export dashboards to JSON:

```bash
# Via API
curl -H "Authorization: Bearer <api-key>" \
  https://grafana.k12n.com/api/dashboards/uid/<uid>

# Via UI
Dashboard → Settings → JSON Model → Copy
```

### Configuration Backup

```bash
# Export persistent volume
kubectl exec -n grafana grafana-<pod> -- \
  tar czf - /var/lib/grafana | \
  kubectl cp grafana-<pod>:/var/lib/grafana - > grafana-backup.tar.gz
```

## Troubleshooting

### Dashboard Not Loading

1. Check TimescaleDB connection:
   ```bash
   kubectl port-forward -n timescaledb svc/timescaledb-primary 5432:5432
   psql -h localhost -U grafana_reader -d homelab
   ```

2. Test query in Explore view
3. Check Grafana logs for SQL errors

### Slow Queries

1. Add indexes to TimescaleDB:
   ```sql
   CREATE INDEX idx_energy_time ON energy_data(time DESC);
   ```

2. Use continuous aggregates for historical data
3. Limit time range in dashboard variables

### Authentication Issues

1. Verify Authentik configuration (if using OAuth)
2. Check secret mounts:
   ```bash
   kubectl get secret -n grafana grafana-admin-password
   ```
3. Reset admin password via CLI:
   ```bash
   kubectl exec -n grafana grafana-<pod> -- \
     grafana-cli admin reset-admin-password <new-password>
   ```

## Related Components

- **Data Source**: [TimescaleDB](../timescaledb) - Time-series database
- **Auth Provider**: Authentik (planned integration)
- **Alternative**: [heatpump-web](../../../applications/heatpump-web) - Custom dashboard SPA
