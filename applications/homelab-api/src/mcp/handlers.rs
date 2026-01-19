use crate::auth::AppState;
use crate::db::DbPool;
use crate::mcp::types::{JsonRpcRequest, ToolCallParams, ToolDefinition};
use crate::repositories::{EnergyRepository, HeatpumpRepository};
use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::{convert::Infallible, time::Duration};
use tokio_stream::{wrappers::IntervalStream, StreamExt};

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
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
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
        "tools/call" => match handle_tool_call(&pool, request.params).await {
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

async fn handle_tool_call(pool: &DbPool, params: Option<Value>) -> Result<Value, ToolError> {
    let params = params.unwrap_or_else(|| json!({}));
    let tool_params: ToolCallParams = serde_json::from_value(params).map_err(|e| ToolError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(json!({ "detail": e.to_string() })),
    })?;

    let arguments = tool_params.arguments;

    let result = match tool_params.name.as_str() {
        "get_server_time" => get_server_time(),
        "energy_hourly_consumption" => energy_hourly_consumption(pool, &arguments).await?,
        "energy_peak_hour_day" => energy_peak_hour_day(pool, &arguments).await?,
        "heatpump_daily_summary" => heatpump_daily_summary(pool, &arguments).await?,
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

fn get_server_time() -> Value {
    let now = Utc::now();
    json!({
        "current_time": now.to_rfc3339(),
        "timestamp": now.timestamp(),
        "year": now.year(),
        "month": now.month(),
        "day": now.day()
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
            description: "Get the current server time. IMPORTANT: Always call this tool first before querying energy or heatpump data to know the correct current date and time. Use the returned 'current_time' field (RFC3339 format) to construct date ranges for other queries. This ensures you query the correct year and dates.".to_string(),
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
