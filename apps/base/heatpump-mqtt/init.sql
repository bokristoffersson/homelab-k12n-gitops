-- TimescaleDB Init Script för Heatpump MQTT Data
-- Detta skript skapar databasen och tabellerna baserat på config.yaml

-- Skapa databasen (om den inte redan finns)
-- OBS: Detta körs som superuser, så vi skapar databasen direkt
CREATE DATABASE timescaledb;

-- Använd databasen
\c timescaledb;

-- Aktivera TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Skapa tabellen för heatpump data baserat på config.yaml
CREATE TABLE IF NOT EXISTS heatpump (
    ts TIMESTAMPTZ NOT NULL,
    outdoor_temp SMALLINT,
    supplyline_temp SMALLINT,
    returnline_temp SMALLINT,
    hotwater_temp SMALLINT,
    brine_out_temp SMALLINT,
    brine_in_temp SMALLINT,
    integral DOUBLE PRECISION,
    flowlinepump_speed SMALLINT,
    brinepump_speed SMALLINT,
    runtime_compressor INTEGER,
    runtime_hotwater INTEGER,
    runtime_3kw INTEGER,
    runtime_6kw INTEGER,
    brinepump_on BOOLEAN,
    compressor_on BOOLEAN,
    flowlinepump_on BOOLEAN,
    hotwater_production BOOLEAN,
    circulation_pump BOOLEAN
);

-- Skapa hypertable för TimescaleDB (ts är partitioneringskolumnen)
SELECT create_hypertable('heatpump', 'ts', if_not_exists => TRUE);

-- Skapa index för bättre prestanda
CREATE INDEX IF NOT EXISTS idx_heatpump_ts ON heatpump (ts DESC);


-- Skapa en kontinuerlig aggragat för daglig sammanfattning (valfritt)
CREATE MATERIALIZED VIEW IF NOT EXISTS heatpump_daily_summary
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 day', ts) AS day,
    -- Runtime-värden: beräkna daglig ökning (sista värdet - första värdet)
    (LAST(runtime_compressor, ts) - FIRST(runtime_compressor, ts)) AS daily_runtime_compressor_increase,
    (LAST(runtime_hotwater, ts) - FIRST(runtime_hotwater, ts)) AS daily_runtime_hotwater_increase,
    (LAST(runtime_3kw, ts) - FIRST(runtime_3kw, ts)) AS daily_runtime_3kw_increase,
    (LAST(runtime_6kw, ts) - FIRST(runtime_6kw, ts)) AS daily_runtime_6kw_increase,
    -- Temperaturer: genomsnitt under dagen
    AVG(outdoor_temp) AS avg_outdoor_temp,
FROM heatpump
GROUP BY day;

-- Visa tabellstrukturen
\d heatpump;

-- Visa hypertable information
SELECT * FROM timescaledb_information.hypertables WHERE hypertable_name = 'heatpump';

-- Visa compression policy
SELECT * FROM timescaledb_information.jobs WHERE hypertable_name = 'heatpump';

PRINT 'TimescaleDB init script completed successfully!';
