-- Migration: 003_fix_energy_actual_calculation
-- Description: Fix energy_hourly to calculate actual energy correctly
--              The previous migration used active_power (W) instead of active_energy (Wh)
--              which resulted in incorrect values.
--
-- Problem: active_power_* columns are instantaneous power in Watts
--          Taking first/last difference gives meaningless values
--
-- Solution: Remove the incorrect "actual" columns since we don't have
--           per-phase energy accumulators in the raw data.
--           Keep only total_energy_kwh which uses active_energy_total correctly.

\c telemetry

-- Drop existing energy_hourly continuous aggregate
DROP MATERIALIZED VIEW IF EXISTS energy_hourly CASCADE;

-- Recreate with ONLY the data we can calculate correctly
-- Note: We only have active_energy_total (cumulative meter reading)
--       There are NO per-phase energy accumulators in energy_consumption table
CREATE MATERIALIZED VIEW energy_hourly
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) AS hour_start,
  (time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) + '1 hour'::interval) AS hour_end,
  -- Total energy consumed during the hour (from cumulative meter reading)
  (last(active_energy_total, time) - first(active_energy_total, time)) / 1000.0 AS total_energy_kwh,
  -- Average power per phase (for informational purposes, not cumulative)
  avg(active_power_l1) / 1000.0 AS avg_power_l1_kw,
  avg(active_power_l2) / 1000.0 AS avg_power_l2_kw,
  avg(active_power_l3) / 1000.0 AS avg_power_l3_kw,
  avg(active_power_total) / 1000.0 AS avg_power_total_kw,
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
INSERT INTO schema_migrations (version, name) VALUES (3, 'fix_energy_actual_calculation')
ON CONFLICT (version) DO NOTHING;
