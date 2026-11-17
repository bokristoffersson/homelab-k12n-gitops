-- Migration: 006_create_power_plugs_tables
-- Description: Create power_plugs and power_plug_schedules tables for Tasmota smart plug management

-- Connect to homelab_settings database
\c homelab_settings

-- Power plugs table (stores Tasmota plug state)
CREATE TABLE IF NOT EXISTS power_plugs (
    plug_id TEXT PRIMARY KEY,              -- Tasmota device ID (e.g., "tasmota_living_room")
    name TEXT NOT NULL,                    -- Friendly name (e.g., "Living Room Lamp")
    status BOOLEAN DEFAULT false,          -- true=ON, false=OFF
    wifi_rssi INTEGER,                     -- Signal strength from telemetry
    uptime_seconds INTEGER,                -- Device uptime
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Daily recurring schedules for power plugs
CREATE TABLE IF NOT EXISTS power_plug_schedules (
    id BIGSERIAL PRIMARY KEY,
    plug_id TEXT NOT NULL REFERENCES power_plugs(plug_id) ON DELETE CASCADE,
    action TEXT NOT NULL CHECK (action IN ('on', 'off')),
    time_of_day TIME NOT NULL,             -- e.g., '08:00:00'
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(plug_id, action, time_of_day)
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_power_plugs_updated_at ON power_plugs (updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_power_plug_schedules_plug_id ON power_plug_schedules (plug_id);
CREATE INDEX IF NOT EXISTS idx_power_plug_schedules_enabled ON power_plug_schedules (enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_power_plug_schedules_time ON power_plug_schedules (time_of_day);

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (6, 'create_power_plugs_tables')
ON CONFLICT (version) DO NOTHING;
