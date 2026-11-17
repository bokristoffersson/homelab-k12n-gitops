-- Migration: 002_fix_indoor_target_temp_type
-- Description: Change indoor_target_temp column from REAL (FLOAT4) to DOUBLE PRECISION (FLOAT8) to match Rust f64 type

\c heatpump_settings

-- Alter the column type from REAL to DOUBLE PRECISION
ALTER TABLE settings ALTER COLUMN indoor_target_temp TYPE DOUBLE PRECISION;

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (2, 'fix_indoor_target_temp_type')
ON CONFLICT (version) DO NOTHING;
