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
    d50_indoor_target_temp DOUBLE PRECISION,
    d51_mode               BIGINT,
    d52_curve              BIGINT,
    d53_curve_min          BIGINT,
    d54_curve_max          BIGINT,
    d55_curve_plus5        BIGINT,
    d56_curve_0            BIGINT,
    d57_curve_minus5       BIGINT,
    d58_heatstop           BIGINT
);

-- Create index on latest_update for querying recent updates
CREATE INDEX IF NOT EXISTS idx_heatpump_settings_latest_update 
ON heatpump_settings(latest_update);
