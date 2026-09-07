-- Migration: 008_add_plug_desired_status
-- Description: Separate desired power state from reported state so a periodic
--              reconciler can re-issue commands until the device converges
--              (level-based, Kubernetes-style). desired_status is NULL until
--              the first command after this migration; the reconciler ignores
--              NULL so existing plugs are unaffected until first touched.

-- Connect to homelab_settings database
\c homelab_settings

ALTER TABLE power_plugs
    ADD COLUMN IF NOT EXISTS desired_status BOOLEAN,
    ADD COLUMN IF NOT EXISTS desired_updated_at TIMESTAMPTZ;

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (8, 'add_plug_desired_status')
ON CONFLICT (version) DO NOTHING;
