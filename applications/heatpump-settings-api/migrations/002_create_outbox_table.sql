-- Migration 002: Create outbox table for transactional outbox pattern
-- This table stores commands to be published to MQTT in a separate process

CREATE TABLE IF NOT EXISTS outbox (
    id BIGSERIAL PRIMARY KEY,

    -- Aggregate identification
    aggregate_type VARCHAR(255) NOT NULL,          -- 'heatpump_setting'
    aggregate_id VARCHAR(255) NOT NULL,            -- device_id

    -- Event details
    event_type VARCHAR(255) NOT NULL,              -- 'setting_update'
    payload JSONB NOT NULL,                        -- {indoor_target_temp: 21.5, mode: 1, ...}

    -- Status tracking
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending → published → confirmed → failed

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,

    -- Error handling
    error_message TEXT,
    retry_count INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3
);

-- Indexes for efficient queries
CREATE INDEX idx_outbox_status ON outbox(status) WHERE status IN ('pending', 'published');
CREATE INDEX idx_outbox_created ON outbox(created_at);
CREATE INDEX idx_outbox_aggregate ON outbox(aggregate_type, aggregate_id);

-- Comments for documentation
COMMENT ON TABLE outbox IS 'Transactional outbox for heatpump setting commands. Commands are inserted atomically with settings updates, then processed asynchronously.';
COMMENT ON COLUMN outbox.status IS 'pending: awaiting publish | published: sent to MQTT | confirmed: heatpump responded | failed: max retries exceeded';
COMMENT ON COLUMN outbox.payload IS 'JSON object containing the setting changes to be applied';
