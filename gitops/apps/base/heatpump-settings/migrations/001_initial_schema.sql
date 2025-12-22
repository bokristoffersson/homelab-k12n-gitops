-- Migration: 001_initial_schema
-- Description: Create initial heatpump_settings database and settings table

-- Create heatpump_settings database if it doesn't exist
SELECT 'CREATE DATABASE heatpump_settings'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'heatpump_settings')\gexec

-- Connect to heatpump_settings database
\c heatpump_settings

-- Create migrations tracking table
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Heatpump settings table (stores latest settings per device)
CREATE TABLE IF NOT EXISTS settings (
  device_id TEXT PRIMARY KEY,
  indoor_target_temp REAL,
  mode INTEGER,
  curve INTEGER,
  curve_min INTEGER,
  curve_max INTEGER,
  curve_plus_5 INTEGER,
  curve_zero INTEGER,
  curve_minus_5 INTEGER,
  heatstop INTEGER,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for updated_at queries
CREATE INDEX IF NOT EXISTS idx_settings_updated_at ON settings (updated_at DESC);

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (1, 'initial_schema')
ON CONFLICT (version) DO NOTHING;
