# Database Schema

## Energy Readings

Stores real-time energy consumption data from smart meters.

```sql
CREATE TABLE energy_readings (
    timestamp TIMESTAMPTZ NOT NULL,
    device_id VARCHAR(50),
    total_energy_kwh DOUBLE PRECISION,
    power_kw DOUBLE PRECISION,
    voltage DOUBLE PRECISION,
    current DOUBLE PRECISION
);

SELECT create_hypertable('energy_readings', 'timestamp');
```

### Indexes

```sql
CREATE INDEX idx_energy_device_time ON energy_readings (device_id, timestamp DESC);
```

## Heatpump Status

Stores heat pump telemetry and status information.

```sql
CREATE TABLE heatpump_status (
    timestamp TIMESTAMPTZ NOT NULL,
    device_id VARCHAR(50),
    mode VARCHAR(20),
    target_temp DOUBLE PRECISION,
    actual_temp DOUBLE PRECISION,
    compressor_running BOOLEAN,
    power_consumption_kw DOUBLE PRECISION
);

SELECT create_hypertable('heatpump_status', 'timestamp');
```

## Temperature Readings

Stores temperature sensor data from various rooms.

```sql
CREATE TABLE temperature_readings (
    timestamp TIMESTAMPTZ NOT NULL,
    sensor_id VARCHAR(50),
    room VARCHAR(50),
    temperature DOUBLE PRECISION,
    humidity DOUBLE PRECISION
);

SELECT create_hypertable('temperature_readings', 'timestamp');
```

## Continuous Aggregates

### Hourly Energy Consumption

```sql
CREATE MATERIALIZED VIEW energy_hourly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', timestamp) AS hour_start,
    device_id,
    AVG(power_kw) AS avg_power_kw,
    SUM(total_energy_kwh) AS total_energy_kwh
FROM energy_readings
GROUP BY hour_start, device_id;
```

### Daily Heatpump Statistics

```sql
CREATE MATERIALIZED VIEW heatpump_daily
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', timestamp) AS day_start,
    device_id,
    AVG(actual_temp) AS avg_temp,
    SUM(power_consumption_kw) AS total_consumption,
    COUNT(*) FILTER (WHERE compressor_running) AS compressor_runtime_count
FROM heatpump_status
GROUP BY day_start, device_id;
```

## Data Retention Policies

```sql
-- Keep raw energy data for 90 days
SELECT add_retention_policy('energy_readings', INTERVAL '90 days');

-- Keep raw heatpump data for 90 days
SELECT add_retention_policy('heatpump_status', INTERVAL '90 days');

-- Keep hourly aggregates for 2 years
SELECT add_retention_policy('energy_hourly', INTERVAL '2 years');
```
