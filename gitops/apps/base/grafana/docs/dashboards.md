# Dashboards

Complete guide to all Grafana dashboards in the homelab.

## Dashboard Organization

Dashboards are organized by data domain:

1. **Energy** - Power consumption and costs
2. **Heat Pump** - HVAC system performance
3. **Temperature** - Environmental monitoring
4. **System** - Infrastructure metrics (planned)

## Energy Monitoring Dashboard

### Overview

Real-time and historical power consumption visualization.

### Panels

#### 1. Current Power (Gauge)
- **Type**: Gauge
- **Query**: Latest power reading
- **Range**: 0-10000W
- **Thresholds**:
  - Green: 0-2000W (normal)
  - Yellow: 2000-4000W (elevated)
  - Red: 4000W+ (high)

```sql
SELECT
  power_w
FROM energy_data
ORDER BY time DESC
LIMIT 1;
```

#### 2. 24-Hour Power Trend (Time Series)
- **Type**: Graph (line chart)
- **Query**: 5-minute average power
- **Time range**: Last 24 hours

```sql
SELECT
  time_bucket('5 minutes', time) AS bucket,
  AVG(power_w) as power
FROM energy_data
WHERE time > NOW() - INTERVAL '24 hours'
GROUP BY bucket
ORDER BY bucket;
```

#### 3. Daily Energy Usage (Bar Chart)
- **Type**: Bar chart
- **Query**: Daily total consumption
- **Time range**: Last 7 days

```sql
SELECT
  date_trunc('day', time) AS day,
  SUM(power_w * INTERVAL '1 second') / 3600000 as kwh
FROM energy_data
WHERE time > NOW() - INTERVAL '7 days'
GROUP BY day
ORDER BY day;
```

#### 4. Cost Estimation (Single Stat)
- **Type**: Stat
- **Query**: Daily cost at 150 öre/kWh
- **Format**: Currency (SEK)

```sql
SELECT
  SUM(power_w * INTERVAL '1 second') / 3600000 * 1.50 as cost_sek
FROM energy_data
WHERE time > date_trunc('day', NOW());
```

#### 5. Peak Usage Times (Heatmap)
- **Type**: Heatmap
- **Query**: Average power by hour and day
- **Time range**: Last 30 days

```sql
SELECT
  extract(hour from time) as hour,
  date_trunc('day', time) as day,
  AVG(power_w) as power
FROM energy_data
WHERE time > NOW() - INTERVAL '30 days'
GROUP BY hour, day
ORDER BY day, hour;
```

### Variables

- `$interval`: Time bucket size (5m, 15m, 1h)
- `$range`: Time range filter
- `$threshold`: Alert threshold (default 5000W)

### Alerts

**High Power Consumption**:
- Condition: power_w > $threshold
- Duration: 5 minutes
- Notification: Email to admin

## Heat Pump Dashboard

### Overview

Monitor heat pump performance, efficiency, and operating status.

### Panels

#### 1. Supply Temperature (Gauge)
- **Type**: Gauge
- **Query**: Current supply temp
- **Range**: 0-60°C

```sql
SELECT supply_temp
FROM heatpump_data
ORDER BY time DESC
LIMIT 1;
```

#### 2. Return Temperature (Gauge)
- **Type**: Gauge
- **Query**: Current return temp
- **Range**: 0-50°C

#### 3. COP (Coefficient of Performance)
- **Type**: Gauge
- **Query**: Current COP
- **Range**: 0-5
- **Thresholds**:
  - Red: <2.0 (poor)
  - Yellow: 2.0-3.0 (fair)
  - Green: >3.0 (good)

```sql
SELECT cop
FROM heatpump_data
ORDER BY time DESC
LIMIT 1;
```

#### 4. Operating Mode (Stat)
- **Type**: Stat
- **Query**: Current mode
- **Values**: heating, cooling, off

```sql
SELECT mode
FROM heatpump_data
ORDER BY time DESC
LIMIT 1;
```

#### 5. Temperature Differential (Time Series)
- **Type**: Graph
- **Query**: Supply vs return temp over 24h

```sql
SELECT
  time,
  supply_temp,
  return_temp,
  supply_temp - return_temp as differential
FROM heatpump_data
WHERE time > NOW() - INTERVAL '24 hours'
ORDER BY time;
```

#### 6. Runtime Hours (Counter)
- **Type**: Stat
- **Query**: Total runtime hours today

```sql
SELECT
  COUNT(*) * INTERVAL '5 minutes' / INTERVAL '1 hour' as hours
FROM heatpump_data
WHERE time > date_trunc('day', NOW())
  AND mode = 'heating';
```

### Variables

- `$mode_filter`: Filter by operating mode
- `$cop_threshold`: Minimum acceptable COP

## Temperature Dashboard

### Overview

Environmental temperature and humidity monitoring from Shelly H&T sensors.

### Panels

#### 1. Current Temperature (Gauge)
- **Type**: Gauge
- **Query**: Latest temperature
- **Range**: -10 to 40°C

```sql
SELECT temperature_c
FROM temperature_data
ORDER BY time DESC
LIMIT 1;
```

#### 2. Current Humidity (Gauge)
- **Type**: Gauge
- **Query**: Latest humidity
- **Range**: 0-100%

```sql
SELECT humidity_percent
FROM temperature_data
ORDER BY time DESC
LIMIT 1;
```

#### 3. Temperature Trend (Time Series)
- **Type**: Graph
- **Query**: Temperature over 24 hours

```sql
SELECT
  time,
  temperature_c
FROM temperature_data
WHERE time > NOW() - INTERVAL '24 hours'
ORDER BY time;
```

#### 4. Humidity Trend (Time Series)
- **Type**: Graph
- **Query**: Humidity over 24 hours

```sql
SELECT
  time,
  humidity_percent
FROM temperature_data
WHERE time > NOW() - INTERVAL '24 hours'
ORDER BY time;
```

#### 5. Daily Statistics (Table)
- **Type**: Table
- **Query**: Min/max/avg for today

```sql
SELECT
  MIN(temperature_c) as min_temp,
  MAX(temperature_c) as max_temp,
  AVG(temperature_c) as avg_temp,
  MIN(humidity_percent) as min_humidity,
  MAX(humidity_percent) as max_humidity,
  AVG(humidity_percent) as avg_humidity
FROM temperature_data
WHERE time > date_trunc('day', NOW());
```

### Alerts

**Temperature Anomaly**:
- Condition: temperature_c < 5°C OR temperature_c > 35°C
- Duration: 10 minutes
- Notification: Warning alert

**High Humidity**:
- Condition: humidity_percent > 70%
- Duration: 1 hour
- Notification: Info alert

## Dashboard Templates

### Creating New Dashboards

1. **Copy existing dashboard**:
   ```bash
   cp dashboards/energy.json dashboards/new-dashboard.json
   ```

2. **Edit JSON structure**:
   - Update `title` and `uid`
   - Modify panel configurations
   - Adjust queries for new data source

3. **Import to Grafana**:
   - UI: + → Import → Upload JSON
   - Or commit to Git for GitOps provisioning

### Query Best Practices

1. **Use time filters**:
   ```sql
   WHERE time > $__timeFrom() AND time < $__timeTo()
   ```

2. **Use time buckets for aggregation**:
   ```sql
   SELECT time_bucket('$interval', time) AS bucket
   ```

3. **Limit result sets**:
   ```sql
   LIMIT 1000
   ```

4. **Use continuous aggregates for historical data**:
   ```sql
   FROM energy_data_5m  -- Pre-aggregated view
   ```

## Panel Types

### Time Series (Graph)
- **Best for**: Trends over time
- **Data**: Multiple time-series
- **Features**: Auto-refresh, zoom, pan

### Gauge
- **Best for**: Current values with thresholds
- **Data**: Single value
- **Features**: Color thresholds, min/max

### Stat
- **Best for**: Key metrics, counters
- **Data**: Single or comparison values
- **Features**: Sparklines, color coding

### Table
- **Best for**: Detailed data, logs
- **Data**: Multiple columns
- **Features**: Sorting, filtering

### Heatmap
- **Best for**: Patterns over time
- **Data**: 2D density data
- **Features**: Color gradients, tooltips

## Variables

Dashboard variables enable dynamic queries and filtering.

### Time Range Variable

```
Name: range
Type: Custom
Values: 1h, 6h, 24h, 7d, 30d
```

Usage: `WHERE time > NOW() - INTERVAL '$range'`

### Interval Variable

```
Name: interval
Type: Interval
Auto: true
Options: 1m, 5m, 15m, 1h, 6h, 1d
```

Usage: `time_bucket('$interval', time)`

### Device Filter

```
Name: device
Type: Query
Query: SELECT DISTINCT sensor_id FROM temperature_data
```

Usage: `WHERE sensor_id = '$device'`

## Dashboard Sharing

### Public Dashboards

Make dashboard public:
1. Dashboard → Settings → General
2. Enable "Make Dashboard Public"
3. Copy public link

### Snapshot Sharing

Create snapshot:
1. Dashboard → Share
2. Snapshot → Publish to snapshots.raintank.io
3. Copy snapshot link (expires after 7 days)

### Export/Import

Export dashboard:
- Dashboard → Settings → JSON Model → Copy JSON

Import dashboard:
- + → Import → Paste JSON or upload file

## Maintenance

### Dashboard Updates

Update via GitOps:
1. Edit JSON in `gitops/apps/base/grafana/dashboards/`
2. Commit and push
3. FluxCD applies changes
4. Grafana reloads dashboards

### Performance Optimization

1. **Reduce panel count**: <10 panels per dashboard
2. **Optimize queries**: Use indexes, aggregates
3. **Adjust refresh rate**: 30s-5m depending on need
4. **Cache query results**: Enable query caching

### Dashboard Cleanup

Remove unused dashboards:
```bash
# Via UI
Dashboard → Settings → Delete

# Via API
curl -X DELETE -H "Authorization: Bearer <token>" \
  https://grafana.k12n.com/api/dashboards/uid/<uid>
```

## Related Resources

- [Grafana Documentation](https://grafana.com/docs/)
- [TimescaleDB Best Practices](../timescaledb/docs/)
- [Query Examples](https://grafana.com/docs/grafana/latest/panels-visualizations/query-transform-data/)
