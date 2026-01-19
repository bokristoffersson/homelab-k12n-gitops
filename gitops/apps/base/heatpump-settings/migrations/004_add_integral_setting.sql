-- Migration: 004_add_integral_setting
-- Description: Add integral_setting (d73) column to settings table
--              This represents the configurable integral setting for the heat pump

-- Add integral_setting column to settings table
ALTER TABLE settings
ADD COLUMN IF NOT EXISTS integral_setting smallint;

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at)
VALUES (4, 'add_integral_setting', NOW())
ON CONFLICT (version) DO NOTHING;
