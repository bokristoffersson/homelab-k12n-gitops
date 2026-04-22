use crate::api::middleware::AuthenticatedUser;
use crate::auth::AppState;
use crate::db::DbPool;
use crate::mcp::types::{JsonRpcRequest, ToolCallParams, ToolDefinition};
use crate::repositories::{EnergyRepository, HeatpumpRepository, TemperatureRepository};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde_json::{json, Value};
use std::{convert::Infallible, time::Duration};
use tokio_stream::{wrappers::IntervalStream, StreamExt};

const SCOPE_READ_ENERGY: &str = "read:energy";
const SCOPE_READ_HEATPUMP: &str = "read:heatpump";
const SCOPE_READ_TEMPERATURE: &str = "read:temperature";
const MCP_ERR_FORBIDDEN: i64 = -32001;

pub async fn sse_handler(
    State((_pool, _config, _validator)): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let ready_event = json!({
        "status": "ready",
        "server": "homelab-api",
    });

    let ready_stream = tokio_stream::iter(vec![Ok(Event::default()
        .event("ready")
        .data(ready_event.to_string()))]);

    let ping_stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(15)))
        .map(|_| Ok(Event::default().event("ping").data("{}")));

    let stream = ready_stream.chain(ping_stream);

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}

pub async fn rpc_handler(
    State((pool, _config, _validator)): State<AppState>,
    user: Option<Extension<AuthenticatedUser>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let scopes: Vec<String> = user.map(|Extension(u)| u.scopes).unwrap_or_default();
    let request: JsonRpcRequest = match serde_json::from_value(payload) {
        Ok(req) => req,
        Err(_) => {
            return Ok(Json(jsonrpc_error(
                Value::Null,
                -32600,
                "Invalid Request",
                None,
            )))
        }
    };

    if let Some(version) = request.jsonrpc.as_deref() {
        if version != "2.0" {
            return Ok(Json(jsonrpc_error(
                request.id.unwrap_or(Value::Null),
                -32600,
                "Invalid Request",
                Some(json!({"detail": "jsonrpc must be 2.0"})),
            )));
        }
    }

    let id = request.id.unwrap_or(Value::Null);

    let result = match request.method.as_str() {
        "initialize" => jsonrpc_ok(id, initialize_result()),
        "tools/list" => jsonrpc_ok(id, tools_list_result()),
        "tools/call" => match handle_tool_call(&pool, &scopes, request.params).await {
            Ok(result) => jsonrpc_ok(id, result),
            Err(err) => jsonrpc_error(id, err.code, err.message, err.data),
        },
        _ => jsonrpc_error(id, -32601, "Method not found", None),
    };

    Ok(Json(result))
}

#[derive(Debug)]
struct ToolError {
    code: i64,
    message: String,
    data: Option<Value>,
}

async fn handle_tool_call(
    pool: &DbPool,
    scopes: &[String],
    params: Option<Value>,
) -> Result<Value, ToolError> {
    let params = params.unwrap_or_else(|| json!({}));
    let tool_params: ToolCallParams = serde_json::from_value(params).map_err(|e| ToolError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(json!({ "detail": e.to_string() })),
    })?;

    let arguments = tool_params.arguments;

    let result = match tool_params.name.as_str() {
        "get_server_time" => get_server_time(),
        "energy_hourly_consumption" => {
            require_tool_scope(scopes, SCOPE_READ_ENERGY, &tool_params.name)?;
            energy_hourly_consumption(pool, &arguments).await?
        }
        "energy_peak_hour_day" => {
            require_tool_scope(scopes, SCOPE_READ_ENERGY, &tool_params.name)?;
            energy_peak_hour_day(pool, &arguments).await?
        }
        "energy_latest" => {
            require_tool_scope(scopes, SCOPE_READ_ENERGY, &tool_params.name)?;
            energy_latest(pool).await?
        }
        "energy_daily_summary" => {
            require_tool_scope(scopes, SCOPE_READ_ENERGY, &tool_params.name)?;
            energy_daily_summary(pool, &arguments).await?
        }
        "energy_monthly_summary" => {
            require_tool_scope(scopes, SCOPE_READ_ENERGY, &tool_params.name)?;
            energy_monthly_summary(pool, &arguments).await?
        }
        "energy_yearly_summary" => {
            require_tool_scope(scopes, SCOPE_READ_ENERGY, &tool_params.name)?;
            energy_yearly_summary(pool, &arguments).await?
        }
        "heatpump_daily_summary" => {
            require_tool_scope(scopes, SCOPE_READ_HEATPUMP, &tool_params.name)?;
            heatpump_daily_summary(pool, &arguments).await?
        }
        "heatpump_cycle_counts" => {
            require_tool_scope(scopes, SCOPE_READ_HEATPUMP, &tool_params.name)?;
            heatpump_cycle_counts(pool, &arguments).await?
        }
        "heatpump_latest" => {
            require_tool_scope(scopes, SCOPE_READ_HEATPUMP, &tool_params.name)?;
            heatpump_latest(pool, &arguments).await?
        }
        "temperature_latest" => {
            require_tool_scope(scopes, SCOPE_READ_TEMPERATURE, &tool_params.name)?;
            temperature_latest(pool, &arguments).await?
        }
        "temperature_all_latest" => {
            require_tool_scope(scopes, SCOPE_READ_TEMPERATURE, &tool_params.name)?;
            temperature_all_latest(pool).await?
        }
        "temperature_history" => {
            require_tool_scope(scopes, SCOPE_READ_TEMPERATURE, &tool_params.name)?;
            temperature_history(pool, &arguments).await?
        }
        _ => {
            return Err(ToolError {
                code: -32601,
                message: "Tool not found".to_string(),
                data: Some(json!({ "tool": tool_params.name })),
            })
        }
    };

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
            }
        ],
        "isError": false
    }))
}

fn require_tool_scope(scopes: &[String], required: &str, tool: &str) -> Result<(), ToolError> {
    if scopes.iter().any(|s| s == required) {
        return Ok(());
    }
    Err(ToolError {
        code: MCP_ERR_FORBIDDEN,
        message: "Forbidden".to_string(),
        data: Some(json!({
            "tool": tool,
            "required_scope": required,
        })),
    })
}

fn get_server_time() -> Value {
    let now = Utc::now();
    // Europe/Stockholm is UTC+1 (CET) or UTC+2 (CEST)
    // For simplicity, we'll use UTC+1 offset. For full DST support, use chrono-tz crate.
    let stockholm_offset = chrono::FixedOffset::east_opt(3600).unwrap(); // UTC+1
    let stockholm_time = now.with_timezone(&stockholm_offset);

    json!({
        "server_time_utc": now.to_rfc3339(),
        "server_time_local": stockholm_time.to_rfc3339(),
        "timezone": "Europe/Stockholm (CET/CEST)",
        "utc_offset_hours": 1,
        "timestamp": now.timestamp(),
        "year": stockholm_time.year(),
        "month": stockholm_time.month(),
        "day": stockholm_time.day(),
        "hour": stockholm_time.hour(),
        "note": "All timestamps in API responses are in UTC. Use server_time_local for current local time."
    })
}

async fn energy_hourly_consumption(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let from = parse_required_datetime(args, "from")?;
    let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);

    let readings = EnergyRepository::get_hourly_history(pool, from, to)
        .await
        .map_err(|e| ToolError {
            code: -32603,
            message: "Database error".to_string(),
            data: Some(json!({ "detail": e.to_string() })),
        })?;

    let hours: Vec<Value> = readings
        .into_iter()
        .map(|r| {
            json!({
                "hour_start": r.hour_start,
                "hour_end": r.hour_end,
                "total_energy_kwh": r.total_energy_kwh,
                "avg_power_l1_kw": r.avg_power_l1_kw,
                "avg_power_l2_kw": r.avg_power_l2_kw,
                "avg_power_l3_kw": r.avg_power_l3_kw,
                "avg_power_total_kw": r.avg_power_total_kw,
                "measurement_count": r.measurement_count
            })
        })
        .collect();

    Ok(json!({
        "from": from,
        "to": to,
        "count": hours.len(),
        "hours": hours
    }))
}

async fn energy_peak_hour_day(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let day = parse_required_datetime(args, "day")?;
    let day_start = day
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": "Invalid day value" })),
        })?;
    let day_start = DateTime::<Utc>::from_naive_utc_and_offset(day_start, Utc);
    let day_end = day_start + chrono::Duration::days(1);

    let peak = EnergyRepository::get_peak_hour_for_day(pool, day_start, day_end)
        .await
        .map_err(|e| ToolError {
            code: -32603,
            message: "Database error".to_string(),
            data: Some(json!({ "detail": e.to_string() })),
        })?;

    let peak_json = peak.map(|r| {
        json!({
            "hour_start": r.hour_start,
            "hour_end": r.hour_end,
            "total_energy_kwh": r.total_energy_kwh
        })
    });

    Ok(json!({
        "day_start": day_start,
        "day_end": day_end,
        "peak": peak_json
    }))
}

async fn heatpump_daily_summary(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let from = parse_required_datetime(args, "from")?;
    let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);
    let device_id = args.get("device_id").and_then(|value| value.as_str());

    let summaries = HeatpumpRepository::get_daily_summary(pool, from, to, device_id)
        .await
        .map_err(|e| ToolError {
            code: -32603,
            message: "Database error".to_string(),
            data: Some(json!({ "detail": e.to_string() })),
        })?;

    let days: Vec<Value> = summaries
        .into_iter()
        .map(|s| {
            json!({
                "day": s.day,
                "daily_runtime_compressor_increase": s.daily_runtime_compressor_increase,
                "daily_runtime_hotwater_increase": s.daily_runtime_hotwater_increase,
                "daily_runtime_3kw_increase": s.daily_runtime_3kw_increase,
                "daily_runtime_6kw_increase": s.daily_runtime_6kw_increase,
                "avg_outdoor_temp": s.avg_outdoor_temp,
                "avg_supplyline_temp": s.avg_supplyline_temp,
                "avg_returnline_temp": s.avg_returnline_temp,
                "avg_hotwater_temp": s.avg_hotwater_temp,
                "avg_brine_out_temp": s.avg_brine_out_temp,
                "avg_brine_in_temp": s.avg_brine_in_temp
            })
        })
        .collect();

    Ok(json!({
        "from": from,
        "to": to,
        "device_id": device_id,
        "count": days.len(),
        "days": days
    }))
}

async fn energy_latest(pool: &DbPool) -> Result<Value, ToolError> {
    let reading = EnergyRepository::get_latest(pool).await.map_err(db_error)?;
    Ok(json!({
        "ts": reading.ts,
        "consumption_total_w": reading.consumption_total_w,
        "consumption_total_actual_w": reading.consumption_total_actual_w,
        "consumption_l1_actual_w": reading.consumption_l1_actual_w,
        "consumption_l2_actual_w": reading.consumption_l2_actual_w,
        "consumption_l3_actual_w": reading.consumption_l3_actual_w,
    }))
}

async fn energy_daily_summary(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let from = parse_required_datetime(args, "from")?;
    let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);
    let summaries = EnergyRepository::get_daily_summary(pool, from, to)
        .await
        .map_err(db_error)?;
    let days: Vec<Value> = summaries
        .into_iter()
        .map(|s| {
            json!({
                "day_start": s.day_start,
                "day_end": s.day_end,
                "energy_consumption_kwh": s.energy_consumption_w,
                "measurement_count": s.measurement_count,
            })
        })
        .collect();
    Ok(json!({ "from": from, "to": to, "count": days.len(), "days": days }))
}

async fn energy_monthly_summary(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let from = parse_required_datetime(args, "from")?;
    let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);
    let summaries = EnergyRepository::get_monthly_summary(pool, from, to)
        .await
        .map_err(db_error)?;
    let months: Vec<Value> = summaries
        .into_iter()
        .map(|s| {
            json!({
                "month_start": s.month_start,
                "month_end": s.month_end,
                "energy_consumption_kwh": s.energy_consumption_w,
                "measurement_count": s.measurement_count,
            })
        })
        .collect();
    Ok(json!({ "from": from, "to": to, "count": months.len(), "months": months }))
}

async fn energy_yearly_summary(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let from = parse_required_datetime(args, "from")?;
    let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);
    let summaries = EnergyRepository::get_yearly_summary(pool, from, to)
        .await
        .map_err(db_error)?;
    let years: Vec<Value> = summaries
        .into_iter()
        .map(|s| {
            json!({
                "year_start": s.year_start,
                "year_end": s.year_end,
                "energy_consumption_kwh": s.energy_consumption_w,
                "measurement_count": s.measurement_count,
            })
        })
        .collect();
    Ok(json!({ "from": from, "to": to, "count": years.len(), "years": years }))
}

async fn heatpump_latest(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let device_id = args.get("device_id").and_then(|v| v.as_str());
    let reading = HeatpumpRepository::get_latest(pool, device_id)
        .await
        .map_err(db_error)?;
    let integral_trend = HeatpumpRepository::get_integral_trend(pool, device_id)
        .await
        .unwrap_or(None);
    Ok(json!({
        "ts": reading.time,
        "device_id": reading.device_id,
        "compressor_on": reading.compressor_on,
        "hotwater_production": reading.hotwater_production,
        "flowlinepump_on": reading.flowlinepump_on,
        "brinepump_on": reading.brinepump_on,
        "aux_heater_3kw_on": reading.aux_heater_3kw_on,
        "aux_heater_6kw_on": reading.aux_heater_6kw_on,
        "outdoor_temp": reading.outdoor_temp,
        "supplyline_temp": reading.supplyline_temp,
        "returnline_temp": reading.returnline_temp,
        "hotwater_temp": reading.hotwater_temp,
        "brine_out_temp": reading.brine_out_temp,
        "brine_in_temp": reading.brine_in_temp,
        "integral": reading.integral,
        "integral_trend": integral_trend,
    }))
}

async fn temperature_latest(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let location = args
        .get("location")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": "Missing field: location" })),
        })?;
    let reading = TemperatureRepository::get_latest_by_location(pool, location)
        .await
        .map_err(db_error)?;
    Ok(match reading {
        Some(r) => json!({
            "time": r.time,
            "location": r.location,
            "temperature_c": r.temperature_c,
            "humidity": r.humidity,
            "battery_percent": r.battery_percent,
        }),
        None => json!(null),
    })
}

async fn temperature_all_latest(pool: &DbPool) -> Result<Value, ToolError> {
    let readings = TemperatureRepository::get_all_latest(pool)
        .await
        .map_err(db_error)?;
    let items: Vec<Value> = readings
        .into_iter()
        .map(|r| {
            json!({
                "time": r.time,
                "location": r.location,
                "temperature_c": r.temperature_c,
                "humidity": r.humidity,
                "battery_percent": r.battery_percent,
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "sensors": items }))
}

async fn temperature_history(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let location_from_arg = args
        .get("location")
        .and_then(|v| v.as_str())
        .or_else(|| args.get("sensor_id").and_then(|v| v.as_str()))
        .ok_or_else(|| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": "Missing field: location (or sensor_id)" })),
        })?;

    // Prefer explicit `from`/`to` if provided; otherwise fall back to an `hours` window.
    let (hours, from_ts, to_ts) = if args.get("from").is_some() {
        let from = parse_required_datetime(args, "from")?;
        let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);
        let delta_hours = ((to - from).num_seconds() / 3600).max(1) as i32;
        (delta_hours, Some(from), Some(to))
    } else {
        let h = args.get("hours").and_then(|v| v.as_i64()).unwrap_or(24) as i32;
        (h, None, None)
    };

    let readings = TemperatureRepository::get_history(pool, location_from_arg, hours)
        .await
        .map_err(db_error)?;

    let filtered: Vec<Value> = readings
        .into_iter()
        .filter(|r| match (from_ts, to_ts) {
            (Some(from), Some(to)) => r.time >= from && r.time < to,
            _ => true,
        })
        .map(|r| {
            json!({
                "time": r.time,
                "device_id": r.device_id,
                "location": r.location,
                "temperature_c": r.temperature_c,
                "humidity": r.humidity,
                "battery_percent": r.battery_percent,
            })
        })
        .collect();

    Ok(json!({
        "location": location_from_arg,
        "from": from_ts,
        "to": to_ts,
        "hours": hours,
        "count": filtered.len(),
        "readings": filtered,
    }))
}

fn db_error(e: crate::error::AppError) -> ToolError {
    ToolError {
        code: -32603,
        message: "Database error".to_string(),
        data: Some(json!({ "detail": e.to_string() })),
    }
}

async fn heatpump_cycle_counts(pool: &DbPool, args: &Value) -> Result<Value, ToolError> {
    let from = parse_required_datetime(args, "from")?;
    let to = parse_optional_datetime(args, "to").unwrap_or_else(Utc::now);
    let device_id = args.get("device_id").and_then(|value| value.as_str());

    let counts = HeatpumpRepository::get_cycle_counts(pool, from, to, device_id)
        .await
        .map_err(|e| ToolError {
            code: -32603,
            message: "Database error".to_string(),
            data: Some(json!({ "detail": e.to_string() })),
        })?;

    Ok(json!({
        "from": from,
        "to": to,
        "device_id": device_id,
        "compressor_starts": counts.compressor_starts,
        "hotwater_starts": counts.hotwater_starts,
        "aux_heater_3kw_starts": counts.aux_3kw_starts,
        "aux_heater_6kw_starts": counts.aux_6kw_starts
    }))
}

fn parse_required_datetime(args: &Value, field: &str) -> Result<DateTime<Utc>, ToolError> {
    let value = args
        .get(field)
        .and_then(|value| value.as_str())
        .ok_or_else(|| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": format!("Missing field: {}", field) })),
        })?;

    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": format!("Invalid RFC3339 for {}: {}", field, e) })),
        })
}

fn parse_optional_datetime(args: &Value, field: &str) -> Option<DateTime<Utc>> {
    args.get(field)
        .and_then(|value| value.as_str())
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn tools_list_result() -> Value {
    let tools = vec![
        ToolDefinition {
            name: "get_server_time".to_string(),
            description: "Get the current server time in both UTC and local time (Europe/Stockholm). CRITICAL: Always call this tool first before querying energy or heatpump data. All API timestamps are in UTC, but users expect local time. Use 'server_time_local' to understand current local date/time, then convert to UTC for queries. Example: If user asks for 'yesterday' and local time is 2026-01-19 14:00 CET, query from 2026-01-17T23:00:00Z to 2026-01-18T23:00:00Z (UTC).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "energy_hourly_consumption".to_string(),
            description: "Get hourly electricity consumption data for a specified time range. Returns total energy consumed (total_energy_kwh) and average power per phase (avg_power_l1/l2/l3_kw) for each hour. Use total_energy_kwh to calculate actual consumption - it represents cumulative meter readings. Average power values show instantaneous load distribution across phases. Use this to analyze energy usage patterns, compare consumption across hours/days, or answer questions about electricity usage.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "RFC3339 timestamp (inclusive). Start of the time range to query."
                    },
                    "to": {
                        "type": "string",
                        "description": "RFC3339 timestamp (exclusive). End of the time range. Defaults to current time if not specified."
                    }
                },
                "required": ["from"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "energy_peak_hour_day".to_string(),
            description: "Find the hour with the highest electricity consumption for a specific day. Returns the hour_start, hour_end, and total_energy_kwh for the peak usage hour. Useful for identifying when energy usage is highest during the day.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "day": {
                        "type": "string",
                        "description": "RFC3339 timestamp representing any time during the day you want to analyze. Only the date portion is used."
                    }
                },
                "required": ["day"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "heatpump_daily_summary".to_string(),
            description: "Get daily heat pump performance summaries including runtime statistics (compressor, hot water, electric heating elements) and temperature averages (outdoor, supply line, return line, hot water, brine). Runtime values are cumulative daily increases in minutes. Use this to track heat pump efficiency, analyze heating patterns, or troubleshoot performance issues.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "RFC3339 timestamp (inclusive). Start of the time range to query."
                    },
                    "to": {
                        "type": "string",
                        "description": "RFC3339 timestamp (exclusive). End of the time range. Defaults to current time if not specified."
                    },
                    "device_id": {
                        "type": "string",
                        "description": "Optional heat pump device identifier to filter results for a specific device."
                    }
                },
                "required": ["from"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "heatpump_cycle_counts".to_string(),
            description: "Count how many times the heat pump compressor, hot water production, and auxiliary heaters started during a time period. Detects state changes from off to on (start events). Use this to analyze compressor cycling frequency, hot water production patterns, and auxiliary heater usage. High cycle counts may indicate short-cycling issues.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "RFC3339 timestamp (inclusive). Start of the time range to query."
                    },
                    "to": {
                        "type": "string",
                        "description": "RFC3339 timestamp (exclusive). End of the time range. Defaults to current time if not specified."
                    },
                    "device_id": {
                        "type": "string",
                        "description": "Optional heat pump device identifier to filter results for a specific device."
                    }
                },
                "required": ["from"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "energy_latest".to_string(),
            description: "Get the latest electricity power reading (instantaneous consumption per phase and cumulative meter values). Requires scope read:energy.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "energy_daily_summary".to_string(),
            description: "Daily energy consumption totals (kWh) for a date range. Requires scope read:energy.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from": {"type": "string", "description": "RFC3339 timestamp (inclusive)."},
                    "to": {"type": "string", "description": "RFC3339 timestamp (exclusive). Defaults to now."}
                },
                "required": ["from"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "energy_monthly_summary".to_string(),
            description: "Monthly energy consumption totals (kWh) for a date range. Requires scope read:energy.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from": {"type": "string", "description": "RFC3339 timestamp (inclusive)."},
                    "to": {"type": "string", "description": "RFC3339 timestamp (exclusive). Defaults to now."}
                },
                "required": ["from"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "energy_yearly_summary".to_string(),
            description: "Yearly energy consumption totals (kWh) for a date range. Requires scope read:energy.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from": {"type": "string", "description": "RFC3339 timestamp (inclusive)."},
                    "to": {"type": "string", "description": "RFC3339 timestamp (exclusive). Defaults to now."}
                },
                "required": ["from"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "heatpump_latest".to_string(),
            description: "Latest heat pump status reading (runtime flags, temperatures, integral and trend). Requires scope read:heatpump.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "device_id": {
                        "type": "string",
                        "description": "Optional heat pump device identifier to filter."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "temperature_latest".to_string(),
            description: "Latest indoor/outdoor temperature reading for a given location. Requires scope read:temperature.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string", "description": "Sensor location label."}
                },
                "required": ["location"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "temperature_all_latest".to_string(),
            description: "Latest reading for every known temperature sensor location. Requires scope read:temperature.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "temperature_history".to_string(),
            description: "Temperature history for a sensor location. Accepts either `hours` (default 24) or an explicit `from`/`to` RFC3339 window. `sensor_id` is accepted as a synonym for `location`. Requires scope read:temperature.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string", "description": "Sensor location label."},
                    "sensor_id": {"type": "string", "description": "Alias for location."},
                    "from": {"type": "string", "description": "Optional RFC3339 start timestamp."},
                    "to": {"type": "string", "description": "Optional RFC3339 end timestamp. Defaults to now when from is set."},
                    "hours": {"type": "integer", "description": "Hours of history when from/to is omitted (default 24)."}
                },
                "additionalProperties": false
            }),
        },
    ];

    json!({ "tools": tools })
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "homelab-api",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn jsonrpc_ok(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Value, code: i64, message: impl Into<String>, data: Option<Value>) -> Value {
    let message = message.into();
    let mut error = json!({
        "code": code,
        "message": message
    });

    if let Some(data) = data {
        if let Some(obj) = error.as_object_mut() {
            obj.insert("data".to_string(), data);
        }
    }

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": error
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_required_datetime_parses_valid_rfc3339() {
        let args = json!({ "from": "2026-01-15T12:00:00Z" });
        let parsed = parse_required_datetime(&args, "from").unwrap();
        assert_eq!(parsed.to_rfc3339(), "2026-01-15T12:00:00+00:00");
    }

    #[test]
    fn parse_required_datetime_rejects_missing_field() {
        let args = json!({});
        let err = parse_required_datetime(&args, "from").unwrap_err();
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("Invalid params"));
    }

    #[test]
    fn parse_optional_datetime_returns_none_for_missing_field() {
        let args = json!({});
        assert!(parse_optional_datetime(&args, "to").is_none());
    }

    #[test]
    fn tools_list_includes_expected_tools() {
        let result = tools_list_result();
        let tools = result
            .get("tools")
            .and_then(|value| value.as_array())
            .expect("tools list");
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(|name| name.as_str()))
            .collect();

        assert!(names.contains(&"energy_hourly_consumption"));
        assert!(names.contains(&"energy_peak_hour_day"));
        assert!(names.contains(&"heatpump_daily_summary"));
        assert!(names.contains(&"energy_latest"));
        assert!(names.contains(&"energy_daily_summary"));
        assert!(names.contains(&"energy_monthly_summary"));
        assert!(names.contains(&"energy_yearly_summary"));
        assert!(names.contains(&"heatpump_latest"));
        assert!(names.contains(&"temperature_latest"));
        assert!(names.contains(&"temperature_all_latest"));
        assert!(names.contains(&"temperature_history"));
    }

    #[test]
    fn require_tool_scope_rejects_missing_scope() {
        let scopes: Vec<String> = vec!["read:heatpump".into()];
        let err = require_tool_scope(&scopes, "read:energy", "energy_latest").unwrap_err();
        assert_eq!(err.code, MCP_ERR_FORBIDDEN);
        assert_eq!(err.message, "Forbidden");
    }

    #[test]
    fn require_tool_scope_accepts_matching_scope() {
        let scopes: Vec<String> = vec!["read:energy".into(), "read:temperature".into()];
        assert!(require_tool_scope(&scopes, "read:energy", "energy_latest").is_ok());
    }

    // `get_server_time` has no scope requirement, so it should succeed with no scopes.
    #[tokio::test]
    async fn get_server_time_is_unscoped() {
        // We can't build a DbPool in unit tests, but we can call the synchronous helper directly.
        let result = get_server_time();
        assert!(result.get("server_time_utc").is_some());
        assert!(result.get("timezone").is_some());
    }

    #[test]
    fn jsonrpc_ok_includes_id_and_result() {
        let payload = jsonrpc_ok(json!(7), json!({"ok": true}));
        assert_eq!(payload.get("jsonrpc").unwrap(), "2.0");
        assert_eq!(payload.get("id").unwrap(), 7);
        assert_eq!(payload.get("result").unwrap(), &json!({"ok": true}));
    }

    #[test]
    fn jsonrpc_error_includes_code_and_message() {
        let payload = jsonrpc_error(json!(1), -32601, "Method not found", None);
        let error = payload.get("error").unwrap();
        assert_eq!(error.get("code").unwrap(), -32601);
        assert_eq!(error.get("message").unwrap(), "Method not found");
    }
}
