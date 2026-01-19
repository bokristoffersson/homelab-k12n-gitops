-- Migration: 004_add_integral_setting
-- Description: Add integral_setting (d73) column to settings table
--              This represents the configurable integral setting for the heat pump
--
-- FluxCD Ordering: This migration runs via postgres-migration Job which has
--                   dependencies configured to run BEFORE the heatpump-settings-api
--                   Deployment restarts. This ensures the column exists when the
--                   application starts and prevents startup failures.

-- Add integral_setting column to settings table
ALTER TABLE settings
ADD COLUMN IF NOT EXISTS integral_setting smallint;

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at)
VALUES (4, 'add_integral_setting', NOW())
ON CONFLICT (version) DO NOTHING;
