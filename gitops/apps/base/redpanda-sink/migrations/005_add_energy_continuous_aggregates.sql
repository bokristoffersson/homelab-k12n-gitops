-- Migration: Add continuous aggregates for hourly energy consumption
-- This migration creates TimescaleDB continuous aggregates matching the pattern
-- of existing aggregates (energy_hourly_summary, energy_daily_summary, etc.)
--
-- The continuous aggregate:
-- - Uses origin '2000-01-01 00:00:00+00' for consistent bucket alignment (no gaps)
-- - Calculates energy consumption as (last - first) to ensure continuity
-- - Automatically refreshes as new data arrives
--
-- Time buckets are aligned to hour boundaries with no gaps:
-- [10:00:00, 11:00:00), [11:00:00, 12:00:00), etc.

-- Check if TimescaleDB is available before creating continuous aggregates
DO $$
BEGIN
  -- Only proceed if TimescaleDB extension is available
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
    
    -- Drop existing continuous aggregate if it exists (for idempotency)
    DROP MATERIALIZED VIEW IF EXISTS energy_hourly CASCADE;
    
    -- Create continuous aggregate for hourly energy consumption
    -- Matches the pattern of existing energy_hourly_summary aggregate
    CREATE MATERIALIZED VIEW energy_hourly
    WITH (timescaledb.continuous) AS
    SELECT
      -- Time bucket aligned to hour boundaries with origin for consistent alignment
      -- Each bucket represents [hour_start, hour_start + 1 hour)
      time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) AS hour_start,
      (time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone) + '01:00:00'::interval) AS hour_end,
      
      -- Energy consumption calculated as difference between last and first reading
      -- This ensures continuity: each hour ends where the next hour begins
      (last(consumption_total_w, ts) - first(consumption_total_w, ts)) AS energy_consumption_w,
      
      -- Energy consumption from actual values (if available)
      (last(consumption_total_actual_w, ts) - first(consumption_total_actual_w, ts)) AS energy_consumption_total_actual_w,
      (last(consumption_L1_actual_w, ts) - first(consumption_L1_actual_w, ts)) AS energy_consumption_L1_actual_w,
      (last(consumption_L2_actual_w, ts) - first(consumption_L2_actual_w, ts)) AS energy_consumption_L2_actual_w,
      (last(consumption_L3_actual_w, ts) - first(consumption_L3_actual_w, ts)) AS energy_consumption_L3_actual_w,
      
      -- Start and end values for the hour (useful for verification)
      first(consumption_total_w, ts) AS hour_start_w,
      last(consumption_total_w, ts) AS hour_end_w,
      first(consumption_total_actual_w, ts) AS hour_start_actual_w,
      last(consumption_total_actual_w, ts) AS hour_end_actual_w,
      
      -- Count of readings per hour
      count(*) AS measurement_count,
      
      -- Total energy per hour (kWh) - calculated from energy consumption
      -- Energy (kWh) = Energy consumption (W) / 1000
      (last(consumption_total_w, ts) - first(consumption_total_w, ts)) / 1000.0 AS total_energy_kwh,
      
      -- Total energy from actual consumption (kWh)
      (last(consumption_total_actual_w, ts) - first(consumption_total_actual_w, ts)) / 1000.0 AS total_energy_actual_kwh,
      (last(consumption_L1_actual_w, ts) - first(consumption_L1_actual_w, ts)) / 1000.0 AS total_energy_L1_actual_kwh,
      (last(consumption_L2_actual_w, ts) - first(consumption_L2_actual_w, ts)) / 1000.0 AS total_energy_L2_actual_kwh,
      (last(consumption_L3_actual_w, ts) - first(consumption_L3_actual_w, ts)) / 1000.0 AS total_energy_L3_actual_kwh
    FROM energy
    GROUP BY (time_bucket('01:00:00'::interval, ts, origin => '2000-01-01 00:00:00+00'::timestamp with time zone))
    WITH NO DATA;
    
    -- Add refresh policy to automatically update the continuous aggregate
    -- Refresh every 5 minutes, keeping last 1 hour of raw data
    -- This ensures the aggregate stays up-to-date as new data arrives
    -- start_offset: How far back to refresh (3 hours)
    -- end_offset: How recent to keep in raw data (1 hour - allows for late-arriving data)
    -- schedule_interval: How often to run the refresh (5 minutes)
    SELECT add_continuous_aggregate_policy('energy_hourly',
      start_offset => INTERVAL '3 hours',
      end_offset => INTERVAL '1 hour',
      schedule_interval => INTERVAL '5 minutes',
      if_not_exists => TRUE
    );
    
    -- Create index on hour_start for faster queries
    CREATE INDEX IF NOT EXISTS idx_energy_hourly_hour_start 
    ON energy_hourly (hour_start DESC);
    
    RAISE NOTICE 'Continuous aggregate energy_hourly created successfully';
    
  ELSE
    RAISE NOTICE 'TimescaleDB extension not found. Skipping continuous aggregate creation.';
  END IF;
END $$;


