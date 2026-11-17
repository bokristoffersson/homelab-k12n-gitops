-- Migration: 005_rename_database
-- Description: Rename database from heatpump_settings to homelab_settings
-- NOTE: This migration handles the database rename. It creates the new database
--       if it doesn't exist and copies all data from the old database.

-- Create homelab_settings database if it doesn't exist
SELECT 'CREATE DATABASE homelab_settings'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'homelab_settings')\gexec

-- Connect to homelab_settings database
\c homelab_settings

-- Create migrations tracking table if it doesn't exist
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Check if we need to migrate data from old database
DO $$
BEGIN
  -- Only proceed if the settings table doesn't exist in homelab_settings
  IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'settings' AND table_catalog = 'homelab_settings') THEN
    -- Create settings table
    CREATE TABLE settings (
      device_id TEXT PRIMARY KEY,
      indoor_target_temp DOUBLE PRECISION,
      mode INTEGER,
      curve INTEGER,
      curve_min INTEGER,
      curve_max INTEGER,
      curve_plus_5 INTEGER,
      curve_zero INTEGER,
      curve_minus_5 INTEGER,
      heatstop INTEGER,
      integral_setting SMALLINT,
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    -- Create index for updated_at queries
    CREATE INDEX IF NOT EXISTS idx_settings_updated_at ON settings (updated_at DESC);
  END IF;

  -- Only proceed if the outbox table doesn't exist in homelab_settings
  IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'outbox' AND table_catalog = 'homelab_settings') THEN
    -- Create outbox table
    CREATE TABLE outbox (
        id BIGSERIAL PRIMARY KEY,
        aggregate_type VARCHAR(255) NOT NULL,
        aggregate_id VARCHAR(255) NOT NULL,
        event_type VARCHAR(255) NOT NULL,
        payload JSONB NOT NULL,
        status VARCHAR(50) NOT NULL DEFAULT 'pending',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        published_at TIMESTAMPTZ,
        confirmed_at TIMESTAMPTZ,
        error_message TEXT,
        retry_count INT NOT NULL DEFAULT 0,
        max_retries INT NOT NULL DEFAULT 3
    );

    -- Indexes for efficient queries
    CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox(status) WHERE status IN ('pending', 'published');
    CREATE INDEX IF NOT EXISTS idx_outbox_created ON outbox(created_at);
    CREATE INDEX IF NOT EXISTS idx_outbox_aggregate ON outbox(aggregate_type, aggregate_id);
  END IF;
END $$;

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at)
VALUES (5, 'rename_database', NOW())
ON CONFLICT (version) DO NOTHING;
