-- Migration: Add heatpump daily summary continuous aggregate
-- This migration creates a TimescaleDB continuous aggregate for daily heatpump statistics.
--
-- The aggregate:
-- - Calculates daily runtime increases for compressor, hotwater, and auxiliary heaters
-- - Computes average temperatures for various heatpump components
-- - Automatically refreshes as new data arrives

-- Check if TimescaleDB is available before creating continuous aggregates
DO $$
BEGIN
  -- Only proceed if TimescaleDB extension is available
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
    
    -- Drop existing continuous aggregate if it exists (for idempotency)
    DROP MATERIALIZED VIEW IF EXISTS heatpump_daily_summary CASCADE;
    
    -- Create continuous aggregate for daily heatpump summary
    CREATE MATERIALIZED VIEW heatpump_daily_summary
    WITH (timescaledb.continuous) AS
    SELECT
      time_bucket('1 day'::interval, ts) AS day,
      (last(runtime_compressor, ts) - first(runtime_compressor, ts)) AS daily_runtime_compressor_increase,
      (last(runtime_hotwater, ts) - first(runtime_hotwater, ts)) AS daily_runtime_hotwater_increase,
      (last(runtime_3kw, ts) - first(runtime_3kw, ts)) AS daily_runtime_3kw_increase,
      (last(runtime_6kw, ts) - first(runtime_6kw, ts)) AS daily_runtime_6kw_increase,
      avg(outdoor_temp) AS avg_outdoor_temp,
      avg(supplyline_temp) AS avg_supplyline_temp,
      avg(returnline_temp) AS avg_returnline_temp,
      avg(hotwater_temp) AS avg_hotwater_temp,
      avg(brine_out_temp) AS avg_brine_out_temp,
      avg(brine_in_temp) AS avg_brine_in_temp
    FROM heatpump
    GROUP BY (time_bucket('1 day'::interval, ts))
    WITH NO DATA;
    
    -- Add refresh policy to automatically update the continuous aggregate
    -- Refresh daily, keeping last 1 day of raw data
    PERFORM add_continuous_aggregate_policy('heatpump_daily_summary',
      start_offset => INTERVAL '3 days',
      end_offset => INTERVAL '1 day',
      schedule_interval => INTERVAL '1 hour',
      if_not_exists => TRUE
    );
    
    -- Create index on day for faster queries
    CREATE INDEX IF NOT EXISTS idx_heatpump_daily_summary_day 
    ON heatpump_daily_summary (day DESC);
    
    RAISE NOTICE 'Continuous aggregate heatpump_daily_summary created successfully';
    
  ELSE
    RAISE NOTICE 'TimescaleDB extension not found. Skipping continuous aggregate creation.';
  END IF;
END $$;
