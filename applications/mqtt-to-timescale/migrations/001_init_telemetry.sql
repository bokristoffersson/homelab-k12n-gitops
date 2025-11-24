CREATE TABLE IF NOT EXISTS telemetry
(
    ts          TIMESTAMPTZ       NOT NULL,
    device_id   TEXT,
    room        TEXT,
    sensor      TEXT,
    location    TEXT,
    flow_temp_c DOUBLE PRECISION,
    return_temp_c DOUBLE PRECISION,
    power_w     BIGINT,
    temperature_c DOUBLE PRECISION,
    humidity_pct DOUBLE PRECISION
);

SELECT create_hypertable('telemetry', 'ts', if_not_exists => TRUE);
