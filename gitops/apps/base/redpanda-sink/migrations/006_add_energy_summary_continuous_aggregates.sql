-- Migration: Add energy summary continuous aggregates (hourly, daily, monthly, yearly)
-- This migration creates TimescaleDB continuous aggregates for energy consumption summaries
-- at different time intervals: hourly, daily, monthly, and yearly.
--
-- These aggregates:
-- - Use origin '2000-01-01 00:00:00+00' for consistent bucket alignment (no gaps)
-- - Calculate energy consumption as (last - first) to ensure continuity
-- - Automatically refresh as new data arrives

-- Check if TimescaleDB is available before creating continuous aggregates
DO $$
BEGIN
  -- Only proceed if TimescaleDB extension is available
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
    
    -- ============================================================================
    -- Energy Hourly Summary Continuous Aggregate
    -- ============================================================================
    DROP MATERIALIZED VIEW IF EXISTS energy_hourly_summary CASCADE;
    
    CREATE MATERIALIZED VIEW energy_hourly_summary
    WITH (timescaledb.continuous) AS
    SELECT
      time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) AS hour_start,
      (time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) + '01:00:00'::interval) AS hour_end,
      (last(consumption_total_w, ts) - first(consumption_total_w, ts)) AS energy_consumption_w,
      first(consumption_total_w, ts) AS hour_start_w,
      last(consumption_total_w, ts) AS hour_end_w,
      count(*) AS measurement_count
    FROM energy
    GROUP BY (time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone))
    WITH NO DATA;
    
    -- Add refresh policy for hourly summary
    PERFORM add_continuous_aggregate_policy('energy_hourly_summary',
      start_offset => INTERVAL '3 hours',
      end_offset => INTERVAL '1 hour',
      schedule_interval => INTERVAL '5 minutes',
      if_not_exists => TRUE
    );
    
    -- Create index on hour_start for faster queries
    CREATE INDEX IF NOT EXISTS idx_energy_hourly_summary_hour_start 
    ON energy_hourly_summary (hour_start DESC);
    
    RAISE NOTICE 'Continuous aggregate energy_hourly_summary created successfully';
    
    -- ============================================================================
    -- Energy Daily Summary Continuous Aggregate
    -- ============================================================================
    DROP MATERIALIZED VIEW IF EXISTS energy_daily_summary CASCADE;
    
    CREATE MATERIALIZED VIEW energy_daily_summary
    WITH (timescaledb.continuous) AS
    SELECT
      time_bucket('1 day'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) AS day_start,
      (time_bucket('1 day'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) + '1 day'::interval) AS day_end,
      (last(consumption_total_w, ts) - first(consumption_total_w, ts)) AS energy_consumption_w,
      first(consumption_total_w, ts) AS day_start_w,
      last(consumption_total_w, ts) AS day_end_w,
      count(*) AS measurement_count
    FROM energy
    GROUP BY (time_bucket('1 day'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone))
    WITH NO DATA;
    
    -- Add refresh policy for daily summary
    PERFORM add_continuous_aggregate_policy('energy_daily_summary',
      start_offset => INTERVAL '3 days',
      end_offset => INTERVAL '1 day',
      schedule_interval => INTERVAL '1 hour',
      if_not_exists => TRUE
    );
    
    -- Create index on day_start for faster queries
    CREATE INDEX IF NOT EXISTS idx_energy_daily_summary_day_start 
    ON energy_daily_summary (day_start DESC);
    
    RAISE NOTICE 'Continuous aggregate energy_daily_summary created successfully';
    
    -- ============================================================================
    -- Energy Monthly Summary Continuous Aggregate
    -- ============================================================================
    DROP MATERIALIZED VIEW IF EXISTS energy_monthly_summary CASCADE;
    
    CREATE MATERIALIZED VIEW energy_monthly_summary
    WITH (timescaledb.continuous) AS
    SELECT
      time_bucket('1 mon'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) AS month_start,
      (time_bucket('1 mon'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) + '1 mon'::interval) AS month_end,
      (last(consumption_total_w, ts) - first(consumption_total_w, ts)) AS energy_consumption_w,
      first(consumption_total_w, ts) AS month_start_w,
      last(consumption_total_w, ts) AS month_end_w,
      count(*) AS measurement_count
    FROM energy
    GROUP BY (time_bucket('1 mon'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone))
    WITH NO DATA;
    
    -- Add refresh policy for monthly summary
    PERFORM add_continuous_aggregate_policy('energy_monthly_summary',
      start_offset => INTERVAL '3 months',
      end_offset => INTERVAL '1 month',
      schedule_interval => INTERVAL '1 day',
      if_not_exists => TRUE
    );
    
    -- Create index on month_start for faster queries
    CREATE INDEX IF NOT EXISTS idx_energy_monthly_summary_month_start 
    ON energy_monthly_summary (month_start DESC);
    
    RAISE NOTICE 'Continuous aggregate energy_monthly_summary created successfully';
    
    -- ============================================================================
    -- Energy Yearly Summary Continuous Aggregate
    -- ============================================================================
    DROP MATERIALIZED VIEW IF EXISTS energy_yearly_summary CASCADE;
    
    CREATE MATERIALIZED VIEW energy_yearly_summary
    WITH (timescaledb.continuous) AS
    SELECT
      time_bucket('1 year'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) AS year_start,
      (time_bucket('1 year'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) + '1 year'::interval) AS year_end,
      (last(consumption_total_w, ts) - first(consumption_total_w, ts)) AS energy_consumption_w,
      first(consumption_total_w, ts) AS year_start_w,
      last(consumption_total_w, ts) AS year_end_w,
      count(*) AS measurement_count
    FROM energy
    GROUP BY (time_bucket('1 year'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone))
    WITH NO DATA;
    
    -- Add refresh policy for yearly summary
    PERFORM add_continuous_aggregate_policy('energy_yearly_summary',
      start_offset => INTERVAL '3 years',
      end_offset => INTERVAL '1 year',
      schedule_interval => INTERVAL '1 week',
      if_not_exists => TRUE
    );
    
    -- Create index on year_start for faster queries
    CREATE INDEX IF NOT EXISTS idx_energy_yearly_summary_year_start 
    ON energy_yearly_summary (year_start DESC);
    
    RAISE NOTICE 'Continuous aggregate energy_yearly_summary created successfully';
    
  ELSE
    RAISE NOTICE 'TimescaleDB extension not found. Skipping continuous aggregate creation.';
  END IF;
END $$;
