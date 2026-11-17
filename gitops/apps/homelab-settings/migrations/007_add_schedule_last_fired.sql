-- Migration: 007_add_schedule_last_fired
-- Description: Track when a plug schedule last fired. The scheduler previously
--              matched an exact minute window on a 60s tick, so a tick drifting
--              across a minute boundary or a pod restart at the scheduled minute
--              silently skipped the schedule for the whole day. With last_fired_at
--              the scheduler instead fires everything that is due-but-unfired
--              today, which is restart-safe and self-healing.

-- Connect to homelab_settings database
\c homelab_settings

ALTER TABLE power_plug_schedules
    ADD COLUMN IF NOT EXISTS last_fired_at TIMESTAMPTZ;

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (7, 'add_schedule_last_fired')
ON CONFLICT (version) DO NOTHING;
