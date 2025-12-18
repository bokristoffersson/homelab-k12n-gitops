-- Migration: Update energy_hourly continuous aggregate refresh policy
--
-- Previous configuration only refreshed a 2-hour window (3 hours ago to 1 hour ago),
-- which meant historical data was never aggregated and only 1-2 hours were available.
--
-- This migration updates the refresh policy to maintain 30 days of historical data,
-- ensuring the dashboard always shows the full 24-hour history chart.
--
-- Changes:
-- - start_offset: 3 hours → 30 days (maintains historical data)
-- - end_offset: 1 hour (unchanged - excludes incomplete current hour)
-- - schedule_interval: 5 minutes (unchanged)

DO $$
BEGIN
  -- Only proceed if TimescaleDB extension is available
  IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN

    -- Check if the continuous aggregate exists
    IF EXISTS (
      SELECT 1 FROM timescaledb_information.continuous_aggregates
      WHERE view_name = 'energy_hourly'
    ) THEN

      -- Remove the old refresh policy
      BEGIN
        PERFORM remove_continuous_aggregate_policy('energy_hourly');
        RAISE NOTICE 'Removed old refresh policy for energy_hourly';
      EXCEPTION
        WHEN OTHERS THEN
          RAISE NOTICE 'No existing policy to remove (this is fine)';
      END;

      -- Add new refresh policy with 30-day start_offset
      PERFORM add_continuous_aggregate_policy('energy_hourly',
        start_offset => INTERVAL '30 days',
        end_offset => INTERVAL '1 hour',
        schedule_interval => INTERVAL '5 minutes'
      );

      RAISE NOTICE 'Updated refresh policy for energy_hourly (start_offset: 30 days)';

      -- Perform an initial refresh to populate historical data
      -- This will aggregate all data from the past 30 days
      PERFORM refresh_continuous_aggregate(
        'energy_hourly',
        NOW() - INTERVAL '30 days',
        NOW()
      );

      RAISE NOTICE 'Refreshed energy_hourly with 30 days of historical data';

    ELSE
      RAISE NOTICE 'Continuous aggregate energy_hourly does not exist. Skipping policy update.';
    END IF;

  ELSE
    RAISE NOTICE 'TimescaleDB extension not found. Skipping policy update.';
  END IF;
END $$;
