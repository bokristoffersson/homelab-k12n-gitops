-- Migration: 006_widen_energy_power_columns
-- Description: Widen SMALLINT power/current columns in energy_consumption to INTEGER.
--
-- Problem: With the power-tariff cap removed, instantaneous power now exceeds the
--          SMALLINT maximum (32767). The energy sink stalled with
--          'value "33700" is out of range for type smallint (22003)', which blocked
--          the timescaledb-energy consumer and stopped all energy data ingestion.
--
-- The energy_hourly continuous aggregate and the energy view reference the
-- active_power_* columns, which blocks ALTER COLUMN TYPE, so they are dropped and
-- recreated. energy_daily_summary (and the monthly/yearly views built on it) only
-- reference active_energy_total (already INTEGER) and are left untouched.

\c telemetry

-- Drop dependents that reference the columns being altered
DROP VIEW IF EXISTS energy;
DROP MATERIALIZED VIEW IF EXISTS energy_hourly CASCADE;

-- Widen power and current columns that can exceed SMALLINT under full load
ALTER TABLE energy_consumption
  ALTER COLUMN active_power_total TYPE INTEGER,
  ALTER COLUMN active_power_l1 TYPE INTEGER,
  ALTER COLUMN active_power_l2 TYPE INTEGER,
  ALTER COLUMN active_power_l3 TYPE INTEGER,
  ALTER COLUMN active_production_total TYPE INTEGER,
  ALTER COLUMN current_l1 TYPE INTEGER,
  ALTER COLUMN current_l2 TYPE INTEGER,
  ALTER COLUMN current_l3 TYPE INTEGER;

-- Recreate the energy view (unchanged definition)
CREATE VIEW energy AS
SELECT
  time AS ts,
  active_energy_total AS consumption_total_w,
  active_power_total::bigint AS consumption_total_actual_w,
  active_power_l1::bigint AS consumption_l1_actual_w,
  active_power_l2::bigint AS consumption_l2_actual_w,
  active_power_l3::bigint AS consumption_l3_actual_w
FROM energy_consumption;

-- Recreate the energy_hourly continuous aggregate (unchanged definition)
CREATE MATERIALIZED VIEW energy_hourly
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) AS hour_start,
  (time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) + '1 hour'::interval) AS hour_end,
  (last(active_energy_total, time) - first(active_energy_total, time)) / 1000.0 AS total_energy_kwh,
  avg(active_power_l1) / 1000.0 AS avg_power_l1_kw,
  avg(active_power_l2) / 1000.0 AS avg_power_l2_kw,
  avg(active_power_l3) / 1000.0 AS avg_power_l3_kw,
  avg(active_power_total) / 1000.0 AS avg_power_total_kw,
  count(*) AS measurement_count
FROM energy_consumption
GROUP BY time_bucket('1 hour'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz);

-- Restore the refresh policy
SELECT add_continuous_aggregate_policy('energy_hourly',
  start_offset => INTERVAL '2 days',
  end_offset => INTERVAL '1 hour',
  schedule_interval => INTERVAL '1 hour',
  if_not_exists => TRUE);

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (6, 'widen_energy_power_columns')
ON CONFLICT (version) DO NOTHING;
