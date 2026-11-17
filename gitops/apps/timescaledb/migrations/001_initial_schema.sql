-- Migration: 001_initial_schema
-- Description: Create initial telemetry database with TimescaleDB hypertables

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create telemetry database if it doesn't exist
SELECT 'CREATE DATABASE telemetry'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'telemetry')\gexec

-- Connect to telemetry database
\c telemetry

-- Enable TimescaleDB extension in telemetry database
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create migrations tracking table
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Energy consumption table (SaveEye meter data)
CREATE TABLE IF NOT EXISTS energy_consumption (
  time TIMESTAMPTZ NOT NULL,
  device_serial TEXT,
  meter_type TEXT,
  wifi_rssi SMALLINT,
  active_power_total SMALLINT,
  active_power_l1 SMALLINT,
  active_power_l2 SMALLINT,
  active_power_l3 SMALLINT,
  active_production_total SMALLINT,
  active_energy_total INTEGER,
  active_energy_production INTEGER,
  voltage_l1 SMALLINT,
  voltage_l2 SMALLINT,
  voltage_l3 SMALLINT,
  current_l1 SMALLINT,
  current_l2 SMALLINT,
  current_l3 SMALLINT,
  power_factor SMALLINT
);

-- Convert to hypertable if not already
SELECT create_hypertable('energy_consumption', 'time', if_not_exists => TRUE);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_energy_device_time ON energy_consumption (device_serial, time DESC);

-- Temperature sensors table (Shelly HT G3 and outdoor sensors)
CREATE TABLE IF NOT EXISTS temperature_sensors (
  time TIMESTAMPTZ NOT NULL,
  device_id TEXT,
  mac_address TEXT,
  location TEXT,
  temperature_c DOUBLE PRECISION,
  temperature_f DOUBLE PRECISION,
  humidity DOUBLE PRECISION,
  wifi_rssi DOUBLE PRECISION,
  battery_voltage DOUBLE PRECISION,
  battery_percent DOUBLE PRECISION,
  external_power DOUBLE PRECISION,
  uptime DOUBLE PRECISION,
  ram_free DOUBLE PRECISION,
  wind_speed_ms DOUBLE PRECISION,
  pressure_hpa DOUBLE PRECISION
);

-- Convert to hypertable if not already
SELECT create_hypertable('temperature_sensors', 'time', if_not_exists => TRUE);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_temp_device_time ON temperature_sensors (device_id, time DESC);
CREATE INDEX IF NOT EXISTS idx_temp_location_time ON temperature_sensors (location, time DESC);

-- Heatpump status table (ThermIQ telemetry)
CREATE TABLE IF NOT EXISTS heatpump_status (
  time TIMESTAMPTZ NOT NULL,
  device_id TEXT,
  outdoor_temp SMALLINT,
  supplyline_temp SMALLINT,
  returnline_temp SMALLINT,
  hotwater_temp SMALLINT,
  brine_out_temp SMALLINT,
  brine_in_temp SMALLINT,
  flowlinepump_speed SMALLINT,
  runtime_compressor INTEGER,
  runtime_hotwater INTEGER,
  runtime_3kw INTEGER,
  runtime_6kw INTEGER,
  indoor_temp SMALLINT,
  brinepump_on BOOLEAN,
  compressor_on BOOLEAN,
  flowlinepump_on BOOLEAN,
  hotwater_production BOOLEAN,
  aux_heater_3kw_on BOOLEAN,
  aux_heater_6kw_on BOOLEAN
);

-- Convert to hypertable if not already
SELECT create_hypertable('heatpump_status', 'time', if_not_exists => TRUE);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_heatpump_device_time ON heatpump_status (device_id, time DESC);

-- Create view for energy consumption API compatibility
CREATE OR REPLACE VIEW energy AS
SELECT
    time AS ts,
    CAST(active_energy_total AS INTEGER) AS consumption_total_w,
    CAST(active_power_total AS BIGINT) AS consumption_total_actual_w,
    CAST(active_power_l1 AS BIGINT) AS consumption_l1_actual_w,
    CAST(active_power_l2 AS BIGINT) AS consumption_l2_actual_w,
    CAST(active_power_l3 AS BIGINT) AS consumption_l3_actual_w
FROM energy_consumption;

-- Create continuous aggregate for hourly energy consumption
-- This calculates energy used during each hour by taking the difference between
-- the last and first cumulative meter readings, ensuring no gaps
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_hourly
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) AS hour_start,
  (time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) + '1 hour'::interval) AS hour_end,
  (last(active_energy_total, time) - first(active_energy_total, time)) / 1000.0 AS total_energy_kwh,
  (last(active_power_total, time) - first(active_power_total, time)) / 1000.0 AS total_energy_actual_kwh,
  (last(active_power_l1, time) - first(active_power_l1, time)) / 1000.0 AS total_energy_L1_actual_kwh,
  (last(active_power_l2, time) - first(active_power_l2, time)) / 1000.0 AS total_energy_L2_actual_kwh,
  (last(active_power_l3, time) - first(active_power_l3, time)) / 1000.0 AS total_energy_L3_actual_kwh,
  count(*) AS measurement_count
FROM energy_consumption
GROUP BY time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz)
WITH NO DATA;

-- Add refresh policy for the continuous aggregate (refresh last 2 days of data every hour)
SELECT add_continuous_aggregate_policy('energy_hourly',
  start_offset => INTERVAL '2 days',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour',
  if_not_exists => TRUE);

-- Add data retention policies (optional - keep last 90 days of raw data)
SELECT add_retention_policy('energy_consumption', INTERVAL '90 days', if_not_exists => TRUE);
SELECT add_retention_policy('temperature_sensors', INTERVAL '90 days', if_not_exists => TRUE);
SELECT add_retention_policy('heatpump_status', INTERVAL '90 days', if_not_exists => TRUE);

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (1, 'initial_schema')
ON CONFLICT (version) DO NOTHING;
