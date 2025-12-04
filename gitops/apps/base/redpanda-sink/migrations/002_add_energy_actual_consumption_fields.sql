-- Migration: Add actual consumption fields to energy_new table
-- This migration adds columns for activeActualConsumption data
-- which includes total and per-phase (L1, L2, L3) actual consumption values
--
-- These fields are extracted from the activeActualConsumption nested object
-- in the MQTT messages and mapped by mqtt-input to Redpanda, then consumed
-- by redpanda-sink and written to this table.

-- Add actual consumption fields to energy_new table
ALTER TABLE energy_new
ADD COLUMN IF NOT EXISTS consumption_total_actual_w BIGINT,
ADD COLUMN IF NOT EXISTS consumption_L1_actual_w BIGINT,
ADD COLUMN IF NOT EXISTS consumption_L2_actual_w BIGINT,
ADD COLUMN IF NOT EXISTS consumption_L3_actual_w BIGINT;

-- Note: Since this is a hypertable, the new columns will automatically
-- be available for new chunks. Existing chunks will have NULL values
-- for these columns, which is acceptable for historical data.

