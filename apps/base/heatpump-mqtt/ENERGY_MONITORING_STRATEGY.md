# Energy Monitoring Strategy

## Overview

This document outlines the strategy for monitoring energy consumption from the MQTT energy sensor and displaying it in Grafana at different time resolutions.

## Architecture

### Database Schema

The `energy` table has a minimal structure:
```sql
CREATE TABLE energy (
    ts TIMESTAMPTZ NOT NULL,
    consumption_total_w INTEGER
);
```

- **ts**: Timestamp when the measurement was taken (minute precision)
- **consumption_total_w**: Cumulative energy consumption counter in watt-hours (Wh)

### Data Flow
1. **MQTT Messages**: Energy sensor publishes messages to `saveeye/telemetry` topic with cumulative counter `activeTotalConsumption`
2. **Storage**: Only `consumption_total_w` (from `activeTotalConsumption.total`) is stored in TimescaleDB `energy` table every minute
3. **Aggregation**: TimescaleDB continuous aggregates pre-calculate energy consumption differences for different resolutions
4. **Visualization**: Grafana displays energy consumption as bar charts showing consumption per time period

### Key Design Decisions

#### 1. Minute-by-Minute Storage
- **Rationale**: Captures granular consumption data while keeping storage manageable
- **Volume**: ~1,440 records/day (1 integer per record), ~43,200/month, ~525,600/year (~2MB/year)
- **Structure**: Simple table with only `ts TIMESTAMPTZ` and `consumption_total_w INTEGER`
- **TimescaleDB Benefits**: Efficient compression and retention policies available

#### 2. Cumulative Counter Approach
- **Storage**: Store only the raw cumulative counter value (`consumption_total_w INTEGER`)
- **Data Source**: From `activeTotalConsumption.total` field in MQTT message
- **Calculation**: Compute energy consumption as `LAST(value) - FIRST(value)` within each time bucket
- **Why**: Avoids manual difference calculation in queries, handles gaps gracefully, minimal storage footprint

#### 3. Continuous Aggregates
TimescaleDB continuous aggregates automatically update and calculate differences:
- **Hourly**: `hourly_energy_consumption_total` - energy consumed per hour
- **Daily**: `daily_energy_consumption_total` - energy consumed per day
- **Monthly**: `monthly_energy_consumption_total` - energy consumed per month
- **Yearly**: `yearly_energy_consumption_total` - energy consumed per year

### Grafana Visualization

The dashboard includes panels for different resolutions:
- **Hourly Energy Consumption**: Bar chart showing kWh per hour
- **Daily Energy Consumption**: Bar chart showing kWh per day
- **Monthly Energy Consumption**: Bar chart showing kWh per month
- **Yearly Energy Consumption**: Bar chart showing kWh per year

Energy consumption is calculated as:
```
Energy (kWh) = (last_value - first_value) / 1000
```
Where:
- Values are in watt-hours (Wh) from the cumulative counter
- Divide by 1000 to convert to kilowatt-hours (kWh)

**Note**: The actual unit depends on your sensor. If the counter is in a different unit, adjust the division accordingly. Common units:
- Watt-hours (Wh): divide by 1000
- Milliwatt-hours (mWh): divide by 1,000,000
- Watt-minutes (W·min): divide by 60,000

## Important Considerations

### 1. Counter Resets
**Problem**: If the device loses power or the counter resets, the difference calculation could show negative values or incorrect spikes.

**Detection**: The continuous aggregates use `LAST(value) - FIRST(value)` which will show:
- **Negative values**: If counter decreased (unlikely but possible)
- **Large spikes**: If counter wrapped around (e.g., overflowed from max to 0)

**Mitigation**:
- Check `measurement_count` in continuous aggregates - low counts suggest gaps
- Filter out negative values in Grafana queries: `WHERE consumption > 0`
- For production, consider adding a data validation layer

### 2. Data Volume
**Current**: Storing every minute is reasonable (~15 MB/year)
**Future Options**:
- TimescaleDB compression policies (compress data older than 7 days)
- Retention policies (delete raw minute data after 90 days, keep aggregates)

### 3. Accuracy
**Assumption**: Each minute's sample represents the full minute's consumption
**Reality**: If updates are less frequent (e.g., every 5 minutes), you need to adjust the calculation

**Handle Sporadic Updates**:
- Count measurements per bucket in continuous aggregates
- Scale consumption: `(diff / measurement_count * expected_count)`

### 4. Missing Data Gaps
If sensor goes offline:
- Continuous aggregates handle gaps automatically
- Consume the total from first reading in the bucket to last
- May understate consumption if the last reading is at the beginning of the period

## Future Enhancements

### RedPanda for Real-Time Data
- **Purpose**: Real-time energy consumption for mobile app via WebSocket
- **Storage**: Separate from TimescaleDB (use RedPanda + Kafka/RedPanda streams)
- **Data**: Last 1-24 hours only, for real-time dashboards
- **Updates**: Stream raw messages to RedPanda, let mobile app subscribe

### Recommended Approach
1. **Keep TimescaleDB for historical data** (minute-by-minute aggregates)
2. **Add RedPanda for recent data** (last 24 hours, real-time updates)
3. **Mobile app**: Query RedPanda via WebSocket for live data
4. **Historical queries**: Query TimescaleDB for older data

## Implementation Steps

1. ✅ Create continuous aggregates for hourly, daily, monthly, yearly
2. ✅ Add Grafana panels for energy consumption at different resolutions
3. ⏳ Apply SQL migrations to create new continuous aggregates
4. ⏳ Configure downsampling to 1-minute intervals (if current frequency is higher)
5. ⏳ Set up compression and retention policies for old data
6. ⏳ Test with real data and verify counter reset handling

## SQL Migration Commands

To apply the new continuous aggregates, run:

```sql
-- Connect to your TimescaleDB instance
\c timescaledb

-- Source the updated schema
\i apps/base/heatpump-mqtt/energy_table.sql
```

If the views already exist, drop them first:
```sql
DROP MATERIALIZED VIEW IF EXISTS energy_hourly_summary CASCADE;
DROP MATERIALIZED VIEW IF EXISTS energy_daily_summary CASCADE;
DROP MATERIALIZED VIEW IF EXISTS energy_monthly_summary CASCADE;
DROP MATERIALIZED VIEW IF EXISTS energy_yearly_summary CASCADE;
```

## Testing

### Verify Counter Handling
Check for counter resets:
```sql
-- Check for negative consumption values
SELECT day, daily_energy_consumption_total 
FROM energy_daily_summary 
WHERE daily_energy_consumption_total < 0;

-- Check for unusually large values (possible overflow)
SELECT day, daily_energy_consumption_total 
FROM energy_daily_summary 
WHERE daily_energy_consumption_total > 1000000;
```

### Verify Data Completeness
```sql
-- Check measurement counts per hour
SELECT hour, measurement_count 
FROM energy_hourly_summary 
WHERE measurement_count < 50  -- Should have ~60 measurements per hour
ORDER BY hour;
```

## References

- [TimescaleDB Continuous Aggregates](https://docs.timescale.com/timescaledb/latest/how-to-guides/continuous-aggregates/)
- [TimescaleDB Compression](https://docs.timescale.com/timescaledb/latest/how-to-guides/compression/)
- [Grafana Time Series Visualization](https://grafana.com/docs/grafana/latest/panels-visualizations/visualizations/time-series/)

