-- Migration: 004_add_integral_to_heatpump
-- Description: Add integral (d25) column to heatpump_status table
--              This represents the current integral value from the heat pump

\c telemetry

-- Add integral column to heatpump_status
ALTER TABLE heatpump_status
ADD COLUMN IF NOT EXISTS integral smallint;

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (4, 'add_integral_to_heatpump')
ON CONFLICT (version) DO NOTHING;
