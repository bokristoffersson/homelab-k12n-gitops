-- Migration: Add energy summary continuous views
-- This migration creates regular views that wrap the continuous aggregates
-- and provide continuous energy consumption calculations using window functions.
--
-- These views ensure continuity between time buckets by using the previous
-- bucket's end value as the current bucket's start value.

-- Check if TimescaleDB is available
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
    
    -- ============================================================================
    -- Energy Hourly Summary Continuous View
    -- ============================================================================
    DROP VIEW IF EXISTS energy_hourly_summary_continuous CASCADE;
    
    CREATE OR REPLACE VIEW energy_hourly_summary_continuous AS
    SELECT
      hour_start,
      hour_end,
      hour_end_w - COALESCE(lag(hour_end_w) OVER (ORDER BY hour_start), hour_start_w) AS energy_consumption_w,
      COALESCE(lag(hour_end_w) OVER (ORDER BY hour_start), hour_start_w) AS hour_start_w,
      hour_end_w,
      measurement_count
    FROM energy_hourly_summary
    ORDER BY hour_start;
    
    RAISE NOTICE 'View energy_hourly_summary_continuous created successfully';
    
    -- ============================================================================
    -- Energy Daily Summary Continuous View
    -- ============================================================================
    DROP VIEW IF EXISTS energy_daily_summary_continuous CASCADE;
    
    CREATE OR REPLACE VIEW energy_daily_summary_continuous AS
    SELECT
      day_start,
      day_end,
      day_end_w - COALESCE(lag(day_end_w) OVER (ORDER BY day_start), day_start_w) AS energy_consumption_w,
      COALESCE(lag(day_end_w) OVER (ORDER BY day_start), day_start_w) AS day_start_w,
      day_end_w,
      measurement_count
    FROM energy_daily_summary
    ORDER BY day_start;
    
    RAISE NOTICE 'View energy_daily_summary_continuous created successfully';
    
    -- ============================================================================
    -- Energy Monthly Summary Continuous View
    -- ============================================================================
    DROP VIEW IF EXISTS energy_monthly_summary_continuous CASCADE;
    
    CREATE OR REPLACE VIEW energy_monthly_summary_continuous AS
    SELECT
      month_start,
      month_end,
      month_end_w - COALESCE(lag(month_end_w) OVER (ORDER BY month_start), month_start_w) AS energy_consumption_w,
      COALESCE(lag(month_end_w) OVER (ORDER BY month_start), month_start_w) AS month_start_w,
      month_end_w,
      measurement_count
    FROM energy_monthly_summary
    ORDER BY month_start;
    
    RAISE NOTICE 'View energy_monthly_summary_continuous created successfully';
    
    -- ============================================================================
    -- Energy Yearly Summary Continuous View
    -- ============================================================================
    DROP VIEW IF EXISTS energy_yearly_summary_continuous CASCADE;
    
    CREATE OR REPLACE VIEW energy_yearly_summary_continuous AS
    SELECT
      year_start,
      year_end,
      year_end_w - COALESCE(lag(year_end_w) OVER (ORDER BY year_start), year_start_w) AS energy_consumption_w,
      COALESCE(lag(year_end_w) OVER (ORDER BY year_start), year_start_w) AS year_start_w,
      year_end_w,
      measurement_count
    FROM energy_yearly_summary
    ORDER BY year_start;
    
    RAISE NOTICE 'View energy_yearly_summary_continuous created successfully';
    
  ELSE
    RAISE NOTICE 'TimescaleDB extension not found. Skipping view creation.';
  END IF;
END $$;
