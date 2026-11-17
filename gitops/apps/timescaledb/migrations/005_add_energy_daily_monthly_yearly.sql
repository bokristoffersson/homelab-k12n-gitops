-- Migration: 005_add_energy_daily_monthly_yearly
-- Description: Add daily, monthly, and yearly energy summary views
--              The hourly continuous aggregate exists but the API also
--              expects daily, monthly, and yearly summaries.
--
-- Daily: continuous aggregate from raw energy_consumption (1-day buckets)
-- Monthly/Yearly: regular views aggregating from the daily continuous aggregate
--                 (TimescaleDB continuous aggregates don't support variable-width
--                  intervals like '1 month' or '1 year')

\c telemetry

-- Daily: continuous aggregate from raw data
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_daily_summary
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 day'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) AS day_start,
  (time_bucket('1 day'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz) + '1 day'::interval) AS day_end,
  (last(active_energy_total, time) - first(active_energy_total, time)) AS energy_consumption_w,
  count(*) AS measurement_count
FROM energy_consumption
GROUP BY time_bucket('1 day'::interval, time, origin => '2000-01-01 00:00:00+00'::timestamptz);

-- Refresh policy: update last 7 days every 6 hours
SELECT add_continuous_aggregate_policy('energy_daily_summary',
  start_offset => INTERVAL '7 days',
  end_offset => INTERVAL '1 day',
  schedule_interval => INTERVAL '6 hours',
  if_not_exists => TRUE);

-- Monthly: regular view aggregating from the daily continuous aggregate
CREATE OR REPLACE VIEW energy_monthly_summary AS
SELECT
  date_trunc('month', day_start) AS month_start,
  (date_trunc('month', day_start) + INTERVAL '1 month') AS month_end,
  SUM(energy_consumption_w) AS energy_consumption_w,
  SUM(measurement_count) AS measurement_count
FROM energy_daily_summary
GROUP BY date_trunc('month', day_start)
ORDER BY month_start;

-- Yearly: regular view aggregating from the daily continuous aggregate
CREATE OR REPLACE VIEW energy_yearly_summary AS
SELECT
  date_trunc('year', day_start) AS year_start,
  (date_trunc('year', day_start) + INTERVAL '1 year') AS year_end,
  SUM(energy_consumption_w) AS energy_consumption_w,
  SUM(measurement_count) AS measurement_count
FROM energy_daily_summary
GROUP BY date_trunc('year', day_start)
ORDER BY year_start;

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (5, 'add_energy_daily_monthly_yearly')
ON CONFLICT (version) DO NOTHING;
