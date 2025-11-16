-- Create missing materialized views for heatpump data
-- Run this script to create the views if they don't exist

-- Aktivera TimescaleDB extension (if not already enabled)
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Skapa en kontinuerlig aggragat för daglig sammanfattning
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
    AVG(supplyline_temp) AS avg_supplyline_temp,
    AVG(returnline_temp) AS avg_returnline_temp,
    AVG(hotwater_temp) AS avg_hotwater_temp,
    AVG(brine_out_temp) AS avg_brine_out_temp,
    AVG(brine_in_temp) AS avg_brine_in_temp
FROM heatpump
GROUP BY day;

-- Verify the view was created
SELECT * FROM timescaledb_information.continuous_aggregates WHERE view_name = 'heatpump_daily_summary';

