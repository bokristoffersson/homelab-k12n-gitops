# Database Schema

The schema is defined by the SQL files in `gitops/apps/timescaledb/migrations/`. This
page summarises it; the migrations are the source of truth.

## Tables

All three telemetry tables are hypertables partitioned on `time`, with a 90-day
retention policy.

### `energy_consumption`

One row per reading from the energy meter.

| Column | Meaning |
|---|---|
| `time` | Reading timestamp |
| `device_serial`, `meter_type` | Meter identity |
| `active_power_total`, `active_power_l1..l3` | Current power draw, total and per phase |
| `active_production_total` | Current production |
| `active_energy_total`, `active_energy_production` | Cumulative meter counters |
| `voltage_l1..l3`, `current_l1..l3`, `power_factor` | Electrical measurements per phase |
| `wifi_rssi` | Meter signal strength |

Index: `(device_serial, time DESC)`.

### `heatpump_status`

One row per reading from the ThermIQ heat pump module.

| Column | Meaning |
|---|---|
| `time`, `device_id` | Reading timestamp and device |
| `outdoor_temp`, `indoor_temp` | Temperatures seen by the heat pump |
| `supplyline_temp`, `returnline_temp`, `hotwater_temp` | Heating circuit and hot water |
| `brine_in_temp`, `brine_out_temp` | Brine circuit |
| `flowlinepump_speed` | Circulation pump speed |
| `runtime_compressor`, `runtime_hotwater`, `runtime_3kw`, `runtime_6kw` | Cumulative runtimes |
| `compressor_on`, `brinepump_on`, `flowlinepump_on`, `hotwater_production`, `aux_heater_3kw_on`, `aux_heater_6kw_on` | Component states |

Migration 004 adds the integral value. Index: `(device_id, time DESC)`.

### `temperature_sensors`

One row per reading from a temperature sensor.

| Column | Meaning |
|---|---|
| `time`, `device_id`, `mac_address`, `location` | Reading timestamp and sensor identity |
| `temperature_c`, `temperature_f`, `humidity` | The measurements |
| `battery_voltage`, `battery_percent`, `external_power` | Power state |
| `wifi_rssi`, `uptime`, `ram_free` | Device health |
| `wind_speed_ms`, `pressure_hpa` | For outdoor sensors |

Indexes: `(device_id, time DESC)` and `(location, time DESC)`.

### `spot_prices`

Electricity spot prices, written by spotprice-api. Hypertable with 30-day retention.
The same migration (007) creates `apns_device_tokens` for the app's push notifications.

### `schema_migrations`

`(version, name, applied_at)`. One row per applied migration.

## Views and aggregates

| Name | Kind | Contents |
|---|---|---|
| `energy` | View | `energy_consumption` with the column names the API uses (`ts`, `consumption_total_w`, …) |
| `energy_hourly` | Continuous aggregate | Energy used per hour, total and per phase |
| `energy_daily_summary` | Continuous aggregate | Energy used per day |
| `energy_monthly_summary`, `energy_yearly_summary` | Views | Built on the daily aggregate |

Continuous aggregates refresh on a schedule set in their migration, so the most recent
bucket can lag the raw data.

## Example queries

```sql
-- Latest energy reading
SELECT * FROM energy ORDER BY ts DESC LIMIT 1;

-- Energy per hour for the last day
SELECT hour_start, total_energy_kwh
FROM energy_hourly
WHERE hour_start > NOW() - INTERVAL '24 hours'
ORDER BY hour_start;

-- Latest reading per temperature sensor
SELECT DISTINCT ON (device_id) device_id, location, time, temperature_c, humidity
FROM temperature_sensors
ORDER BY device_id, time DESC;
```
