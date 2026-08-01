-- Migration: 001_initial_schema
-- Description: Create the gasa database and the slots table for driving-practice bookings.
-- All statements are idempotent: this script re-runs on every Flux sync.

-- Create gasa database if it doesn't exist
SELECT 'CREATE DATABASE gasa'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'gasa')\gexec

\c gasa

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS slots (
  id BIGSERIAL PRIMARY KEY,
  starts_at TIMESTAMPTZ NOT NULL,
  ends_at TIMESTAMPTZ NOT NULL,
  note TEXT,
  created_by TEXT NOT NULL,
  booked_by_username TEXT,
  booked_by_email TEXT,
  booked_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT slots_time_valid CHECK (ends_at > starts_at)
);

CREATE INDEX IF NOT EXISTS idx_slots_starts_at ON slots (starts_at);

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (1, 'initial_schema')
ON CONFLICT (version) DO NOTHING;
