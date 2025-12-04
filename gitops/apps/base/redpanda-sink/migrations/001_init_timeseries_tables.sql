-- Timeseries tables (hypertables for TimescaleDB)
-- These tables store time-series data efficiently
-- Schema matches the pipeline configuration in configmap.yaml
--
-- NOTE: Tables use "_new" suffix for parallel operation during migration from heatpump-mqtt.
-- After migration is complete and old system is removed, these tables will be renamed
-- to remove the "_new" suffix (heatpump_new -> heatpump, etc.)

-- Heatpump data table (with _new suffix for parallel operation)
CREATE TABLE IF NOT EXISTS heatpump_new
(
    ts                  TIMESTAMPTZ       NOT NULL,
    device_id           TEXT,
    room                TEXT,
    outdoor_temp        DOUBLE PRECISION,
    supplyline_temp     DOUBLE PRECISION,
    returnline_temp     DOUBLE PRECISION,
    hotwater_temp       BIGINT,
    brine_out_temp      BIGINT,
    brine_in_temp       BIGINT,
    integral            BIGINT,
    flowlinepump_speed  BIGINT,
    brinepump_speed     BIGINT,
    runtime_compressor  BIGINT,
    runtime_hotwater    BIGINT,
    runtime_3kw         BIGINT,
    runtime_6kw         BIGINT,
    brinepump_on        BOOLEAN,
    compressor_on       BOOLEAN,
    flowlinepump_on     BOOLEAN,
    hotwater_production BOOLEAN,
    circulation_pump    BOOLEAN,
    aux_heater_3kw_on   BOOLEAN,
    aux_heater_6kw_on   BOOLEAN
);

-- Convert to hypertable if TimescaleDB is available
SELECT create_hypertable('heatpump_new', 'ts', if_not_exists => TRUE);

-- Telemetry table for sensor data (with _new suffix for parallel operation)
CREATE TABLE IF NOT EXISTS telemetry_new
(
    ts          TIMESTAMPTZ       NOT NULL,
    sensor       TEXT,
    location     TEXT,
    temperature_c DOUBLE PRECISION,
    humidity_pct DOUBLE PRECISION
);

-- Convert to hypertable if TimescaleDB is available
SELECT create_hypertable('telemetry_new', 'ts', if_not_exists => TRUE);

-- Energy consumption table (with _new suffix for parallel operation)
CREATE TABLE IF NOT EXISTS energy_new
(
    ts                 TIMESTAMPTZ       NOT NULL,
    consumption_total_w DOUBLE PRECISION,
    consumption_l1_w    DOUBLE PRECISION,
    consumption_l2_w    DOUBLE PRECISION,
    consumption_l3_w    DOUBLE PRECISION
);

-- Convert to hypertable if TimescaleDB is available
SELECT create_hypertable('energy_new', 'ts', if_not_exists => TRUE);
