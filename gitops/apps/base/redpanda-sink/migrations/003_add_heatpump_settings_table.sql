-- Migration: Add heatpump_settings static table
-- This table stores heatpump configuration settings (d50-d58) as static data
-- Settings are updated via upsert based on device_id (extracted from Client_Name field)
--
-- Static table for heatpump settings
-- Uses device_id as primary key for upsert operations
CREATE TABLE IF NOT EXISTS heatpump_settings
(
    device_id              TEXT PRIMARY KEY,
    latest_update          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    indoor_target_temp     DOUBLE PRECISION,
    mode                   BIGINT,
    curve                  BIGINT,
    curve_min              BIGINT,
    curve_max              BIGINT,
    curve_plus5            BIGINT,
    curve_0                BIGINT,
    curve_minus5           BIGINT,
    heatstop               BIGINT
);

-- Create index on latest_update for querying recent updates
CREATE INDEX IF NOT EXISTS idx_heatpump_settings_latest_update 
ON heatpump_settings(latest_update);
