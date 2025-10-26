-- Energy table creation script for TimescaleDB
-- Based on config.yaml energy pipeline configuration
-- Only stores consumption_total_w (cumulative energy counter) every minute

-- Aktivera TimescaleDB extension (if not already enabled)
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Skapa tabellen för energy data baserat på config.yaml
-- Only stores consumption_total_w INTEGER from activeTotalConsumption.total
CREATE TABLE IF NOT EXISTS energy (
    ts TIMESTAMPTZ NOT NULL,
    consumption_total_w INTEGER
);

-- Skapa hypertable för TimescaleDB (ts är partitioneringskolumnen)
SELECT create_hypertable('energy', 'ts', if_not_exists => TRUE);

-- Skapa index för bättre prestanda
CREATE INDEX IF NOT EXISTS idx_energy_ts ON energy (ts DESC);

-- Skapa en kontinuerlig aggragat för daglig sammanfattning
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_daily_summary
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 day', ts) AS day,
    -- Genomsnittlig konsumtion under dagen
    AVG(consumption_total_w) AS avg_consumption_total_w,
    MAX(consumption_total_w) AS max_consumption_total_w,
    MIN(consumption_total_w) AS min_consumption_total_w,
    -- Total energy consumption: difference between first and last reading in the day
    -- For cumulative counters, this gives us the total energy consumed during the day
    (LAST(consumption_total_w, ts) - FIRST(consumption_total_w, ts)) AS daily_energy_consumption_total,
    FIRST(consumption_total_w, ts) AS day_start_total,
    LAST(consumption_total_w, ts) AS day_end_total,
    -- Antal mätpunkter
    COUNT(*) AS measurement_count
FROM energy
GROUP BY day;

-- Skapa en kontinuerlig aggragat för timvis sammanfattning
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_hourly_summary
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', ts) AS hour,
    -- Genomsnittlig konsumtion per timme
    AVG(consumption_total_w) AS avg_consumption_total_w,
    MAX(consumption_total_w) AS max_consumption_total_w,
    MIN(consumption_total_w) AS min_consumption_total_w,
    -- Total energy consumption: difference between first and last reading in the hour
    (LAST(consumption_total_w, ts) - FIRST(consumption_total_w, ts)) AS hourly_energy_consumption_total,
    FIRST(consumption_total_w, ts) AS hour_start_total,
    LAST(consumption_total_w, ts) AS hour_end_total,
    -- Antal mätpunkter per timme
    COUNT(*) AS measurement_count
FROM energy
GROUP BY hour;

-- Skapa en kontinuerlig aggragat för månadsvis sammanfattning
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_monthly_summary
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 month', ts) AS month,
    -- Genomsnittlig konsumtion under månaden
    AVG(consumption_total_w) AS avg_consumption_total_w,
    MAX(consumption_total_w) AS max_consumption_total_w,
    MIN(consumption_total_w) AS min_consumption_total_w,
    -- Total energy consumption: difference between first and last reading in the month
    (LAST(consumption_total_w, ts) - FIRST(consumption_total_w, ts)) AS monthly_energy_consumption_total,
    FIRST(consumption_total_w, ts) AS month_start_total,
    LAST(consumption_total_w, ts) AS month_end_total,
    -- Antal mätpunkter
    COUNT(*) AS measurement_count
FROM energy
GROUP BY month;

-- Skapa en kontinuerlig aggragat för årlig sammanfattning
CREATE MATERIALIZED VIEW IF NOT EXISTS energy_yearly_summary
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 year', ts) AS year,
    -- Genomsnittlig konsumtion under året
    AVG(consumption_total_w) AS avg_consumption_total_w,
    MAX(consumption_total_w) AS max_consumption_total_w,
    MIN(consumption_total_w) AS min_consumption_total_w,
    -- Total energy consumption: difference between first and last reading in the year
    (LAST(consumption_total_w, ts) - FIRST(consumption_total_w, ts)) AS yearly_energy_consumption_total,
    FIRST(consumption_total_w, ts) AS year_start_total,
    LAST(consumption_total_w, ts) AS year_end_total,
    -- Antal mätpunkter
    COUNT(*) AS measurement_count
FROM energy
GROUP BY year;

-- Visa tabellstrukturen
\d energy;

-- Visa hypertable information
SELECT * FROM timescaledb_information.hypertables WHERE hypertable_name = 'energy';

-- Visa materialized views
SELECT * FROM timescaledb_information.continuous_aggregates WHERE view_name IN ('energy_hourly_summary', 'energy_daily_summary', 'energy_monthly_summary', 'energy_yearly_summary');

PRINT 'Energy table creation script completed successfully!';
