-- Migration: Rename heatpump_settings table columns
-- This migration removes the d5X_ prefix from column names, keeping only the part after the underscore
-- 
-- Column renames:
--   d50_indoor_target_temp -> indoor_target_temp
--   d51_mode -> mode
--   d52_curve -> curve
--   d53_curve_min -> curve_min
--   d54_curve_max -> curve_max
--   d55_curve_plus5 -> curve_plus5
--   d56_curve_0 -> curve_0
--   d57_curve_minus5 -> curve_minus5
--   d58_heatstop -> heatstop

-- Rename columns in heatpump_settings table
-- This migration is idempotent: it only renames columns if they exist with the old names
-- If the table was created fresh with new column names (migration 003), this will be a no-op

DO $$
BEGIN
  -- Check if old column exists before renaming
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd50_indoor_target_temp'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d50_indoor_target_temp TO indoor_target_temp;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd51_mode'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d51_mode TO mode;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd52_curve'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d52_curve TO curve;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd53_curve_min'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d53_curve_min TO curve_min;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd54_curve_max'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d54_curve_max TO curve_max;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd55_curve_plus5'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d55_curve_plus5 TO curve_plus5;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd56_curve_0'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d56_curve_0 TO curve_0;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd57_curve_minus5'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d57_curve_minus5 TO curve_minus5;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'heatpump_settings' 
    AND column_name = 'd58_heatstop'
  ) THEN
    ALTER TABLE heatpump_settings RENAME COLUMN d58_heatstop TO heatstop;
  END IF;
END $$;
