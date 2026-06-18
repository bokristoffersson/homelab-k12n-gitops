-- Migration: 007_add_spot_prices
-- Description: Store Nord Pool day-ahead spot prices (SE3) and APNs device tokens
--              for the spotprice-api service.

\c telemetry

-- Day-ahead spot prices. One row per delivery period (hour) per area.
-- Negative prices occur, so DOUBLE PRECISION is required.
CREATE TABLE IF NOT EXISTS spot_prices (
  time TIMESTAMPTZ NOT NULL,
  delivery_area TEXT NOT NULL,
  currency TEXT NOT NULL,
  price_per_mwh DOUBLE PRECISION NOT NULL,
  price_per_kwh DOUBLE PRECISION NOT NULL,
  -- Nord Pool's publish timestamp; not always present in the response.
  source_updated_at TIMESTAMPTZ,
  fetched_at TIMESTAMPTZ NOT NULL
);

-- Convert to hypertable (partitioned on time).
SELECT create_hypertable('spot_prices', 'time', if_not_exists => TRUE);

-- Unique per (area, time) so the fetcher can UPSERT idempotently.
-- The partitioning column (time) must be part of the unique index.
CREATE UNIQUE INDEX IF NOT EXISTS idx_spot_prices_area_time
  ON spot_prices (delivery_area, time);

-- Only today's and tomorrow's prices matter; keep a short history.
SELECT add_retention_policy('spot_prices', INTERVAL '30 days', if_not_exists => TRUE);

-- Registered APNs device tokens (regular table, not a hypertable).
CREATE TABLE IF NOT EXISTS apns_device_tokens (
  token TEXT PRIMARY KEY,
  user_sub TEXT,
  environment TEXT NOT NULL CHECK (environment IN ('sandbox', 'production')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Record migration
INSERT INTO schema_migrations (version, name)
VALUES (7, 'add_spot_prices')
ON CONFLICT (version) DO NOTHING;
