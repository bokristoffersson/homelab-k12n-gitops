-- Migration: 002_fix_energy_hourly
-- Description: Fix energy_hourly column names to match API expectations

\c telemetry

-- Drop existing energy_hourly continuous aggregate if it exists
DROP MATERIALIZED VIEW IF EXISTS energy_hourly CASCADE;

-- Recreate with correct column names (L1, L2, L3 capitalized)
CREATE MATERIALIZED VIEW energy_hourly
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
GROUP BY time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz);

-- Add refresh policy for the continuous aggregate
SELECT add_continuous_aggregate_policy('energy_hourly',
  start_offset => INTERVAL '2 days',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour',
  if_not_exists => TRUE);

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (2, 'fix_energy_hourly')
ON CONFLICT (version) DO NOTHING;
