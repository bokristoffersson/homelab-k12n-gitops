use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    Extension, Json,
};
use serde_json::{json, Value};
use std::{convert::Infallible, time::Duration};
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::api::handlers::AppState;
use crate::auth::AuthContext;
use crate::error::AppError;
use crate::mcp::types::{JsonRpcRequest, ToolCallParams, ToolDefinition};
use crate::repositories::plugs::PowerPlugToggle;

pub async fn sse_handler() -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let ready_event = json!({
        "status": "ready",
        "server": "homelab-settings-api",
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
    State(state): State<AppState>,
    auth: Option<Extension<AuthContext>>,
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
    let auth_ctx = auth.map(|Extension(ctx)| ctx);

    let result = match request.method.as_str() {
        "initialize" => jsonrpc_ok(id, initialize_result()),
        "tools/list" => jsonrpc_ok(id, tools_list_result()),
        "tools/call" => match handle_tool_call(&state, auth_ctx.as_ref(), request.params).await {
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

impl ToolError {
    fn forbidden(scope: &str) -> Self {
        Self {
            code: -32000,
            message: "Forbidden".to_string(),
            data: Some(json!({ "detail": format!("Missing required scope: {}", scope) })),
        }
    }
}

fn require_scope(auth: Option<&AuthContext>, scope: &str) -> Result<(), ToolError> {
    match auth {
        Some(ctx) if ctx.has_scope(scope) => Ok(()),
        Some(_) => Err(ToolError::forbidden(scope)),
        // Mirrors middleware behavior: when JWT validation is disabled, no AuthContext is
        // attached and scope checks are skipped (trust the network perimeter).
        None => Ok(()),
    }
}

fn map_app_error(err: AppError) -> ToolError {
    match err {
        AppError::NotFound(msg) => ToolError {
            code: -32001,
            message: "Not found".to_string(),
            data: Some(json!({ "detail": msg })),
        },
        AppError::InvalidInput(msg) => ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": msg })),
        },
        other => ToolError {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(json!({ "detail": other.to_string() })),
        },
    }
}

async fn handle_tool_call(
    state: &AppState,
    auth: Option<&AuthContext>,
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
        "list_plugs" => {
            require_scope(auth, "read:plugs")?;
            list_plugs(state).await?
        }
        "get_plug" => {
            require_scope(auth, "read:plugs")?;
            get_plug(state, &arguments).await?
        }
        "toggle_plug" => {
            require_scope(auth, "write:plugs")?;
            toggle_plug(state, &arguments).await?
        }
        "list_heatpump_settings" => {
            require_scope(auth, "read:settings")?;
            list_heatpump_settings(state).await?
        }
        "get_heatpump_settings" => {
            require_scope(auth, "read:settings")?;
            get_heatpump_settings(state, &arguments).await?
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

async fn list_plugs(state: &AppState) -> Result<Value, ToolError> {
    let plugs = state
        .plugs_repository
        .get_all()
        .await
        .map_err(map_app_error)?;
    let plugs: Vec<Value> = plugs.into_iter().map(plug_to_json).collect();
    Ok(json!({ "count": plugs.len(), "plugs": plugs }))
}

async fn get_plug(state: &AppState, args: &Value) -> Result<Value, ToolError> {
    let plug_id = require_string_arg(args, "plug_id")?;
    let plug = state
        .plugs_repository
        .get_by_id(plug_id)
        .await
        .map_err(map_app_error)?;
    Ok(plug_to_json(plug))
}

async fn toggle_plug(state: &AppState, args: &Value) -> Result<Value, ToolError> {
    let plug_id = require_string_arg(args, "plug_id")?;
    let status = args
        .get("status")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": "Missing or non-boolean field: status" })),
        })?;

    // Mirrors the PATCH endpoint exactly: open a transaction, update plug status,
    // insert an outbox command, commit atomically.
    let mut tx = state.pool.begin().await.map_err(|e| ToolError {
        code: -32603,
        message: "Internal error".to_string(),
        data: Some(json!({ "detail": e.to_string() })),
    })?;

    let plug =
        crate::repositories::plugs::PlugsRepository::update_status_in_tx(&mut tx, plug_id, status)
            .await
            .map_err(map_app_error)?;

    let outbox_entry = crate::repositories::outbox::OutboxRepository::insert_plug_command_in_tx(
        &mut tx, plug_id, status,
    )
    .await
    .map_err(map_app_error)?;

    tx.commit().await.map_err(|e| ToolError {
        code: -32603,
        message: "Internal error".to_string(),
        data: Some(json!({ "detail": e.to_string() })),
    })?;

    // PowerPlugToggle exists for request parsing on the REST side; include the request
    // shape in the response so MCP callers see the same semantics.
    let requested = PowerPlugToggle { status };

    Ok(json!({
        "plug": plug_to_json(plug),
        "outbox_id": outbox_entry.id,
        "outbox_status": outbox_entry.status,
        "requested": { "status": requested.status },
    }))
}

async fn list_heatpump_settings(state: &AppState) -> Result<Value, ToolError> {
    let settings = state.repository.get_all().await.map_err(map_app_error)?;
    let items: Vec<Value> = settings.into_iter().map(setting_to_json).collect();
    Ok(json!({ "count": items.len(), "settings": items }))
}

async fn get_heatpump_settings(state: &AppState, args: &Value) -> Result<Value, ToolError> {
    let device_id = require_string_arg(args, "device_id")?;
    let setting = state
        .repository
        .get_by_device_id(device_id)
        .await
        .map_err(map_app_error)?;
    Ok(setting_to_json(setting))
}

fn plug_to_json(plug: crate::repositories::plugs::PowerPlug) -> Value {
    json!({
        "plug_id": plug.plug_id,
        "name": plug.name,
        "status": plug.status,
        "wifi_rssi": plug.wifi_rssi,
        "uptime_seconds": plug.uptime_seconds,
        "updated_at": plug.updated_at,
    })
}

fn setting_to_json(setting: crate::repositories::settings::Setting) -> Value {
    json!({
        "device_id": setting.device_id,
        "indoor_target_temp": setting.indoor_target_temp,
        "mode": setting.mode,
        "curve": setting.curve,
        "curve_min": setting.curve_min,
        "curve_max": setting.curve_max,
        "curve_plus_5": setting.curve_plus_5,
        "curve_zero": setting.curve_zero,
        "curve_minus_5": setting.curve_minus_5,
        "heatstop": setting.heatstop,
        "integral_setting": setting.integral_setting,
        "updated_at": setting.updated_at,
    })
}

fn require_string_arg<'a>(args: &'a Value, field: &str) -> Result<&'a str, ToolError> {
    args.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({ "detail": format!("Missing field: {}", field) })),
        })
}

fn tools_list_result() -> Value {
    let tools = vec![
        ToolDefinition {
            name: "list_plugs".to_string(),
            description: "List all power plugs with their current status, name, and telemetry.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "get_plug".to_string(),
            description: "Get a single power plug by id.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plug_id": { "type": "string", "description": "Plug identifier." }
                },
                "required": ["plug_id"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "toggle_plug".to_string(),
            description: "Turn a power plug on (true) or off (false). Uses the same transactional outbox pattern as the REST PATCH endpoint; returns an outbox_id that can be polled for command confirmation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plug_id": { "type": "string", "description": "Plug identifier." },
                    "status": { "type": "boolean", "description": "Desired plug state: true = on, false = off." }
                },
                "required": ["plug_id", "status"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "list_heatpump_settings".to_string(),
            description: "List current heatpump settings for all devices (read-only).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "get_heatpump_settings".to_string(),
            description: "Get current heatpump settings for a specific device (read-only). This service deliberately exposes no tool for writing heatpump settings.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "device_id": { "type": "string", "description": "Heatpump device identifier." }
                },
                "required": ["device_id"],
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
            "name": "homelab-settings-api",
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
    use crate::auth::Claims;

    fn jwt_ctx(scope: &str) -> AuthContext {
        AuthContext::Jwt(Claims {
            sub: "agent".into(),
            exp: 0,
            iat: None,
            iss: None,
            email: None,
            scope: Some(scope.to_string()),
        })
    }

    #[test]
    fn require_scope_allows_match() {
        let ctx = jwt_ctx("read:plugs write:plugs");
        assert!(require_scope(Some(&ctx), "read:plugs").is_ok());
        assert!(require_scope(Some(&ctx), "write:plugs").is_ok());
    }

    #[test]
    fn require_scope_rejects_missing() {
        let ctx = jwt_ctx("read:plugs");
        let err = require_scope(Some(&ctx), "write:plugs").unwrap_err();
        assert_eq!(err.code, -32000);
        assert!(err.message.contains("Forbidden"));
    }

    #[test]
    fn require_scope_allows_proxy() {
        let ctx = AuthContext::Proxy {
            user: "user".into(),
        };
        assert!(require_scope(Some(&ctx), "write:plugs").is_ok());
    }

    #[test]
    fn require_scope_allows_missing_context() {
        assert!(require_scope(None, "write:plugs").is_ok());
    }

    #[test]
    fn tools_list_includes_expected_tools() {
        let result = tools_list_result();
        let tools = result
            .get("tools")
            .and_then(|v| v.as_array())
            .expect("tools list");
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(names.contains(&"list_plugs"));
        assert!(names.contains(&"get_plug"));
        assert!(names.contains(&"toggle_plug"));
        assert!(names.contains(&"list_heatpump_settings"));
        assert!(names.contains(&"get_heatpump_settings"));
        assert!(!names.iter().any(|n| n.starts_with("update_heatpump")
            || n.starts_with("set_heatpump")
            || n.starts_with("patch_heatpump")));
    }

    #[test]
    fn require_string_arg_rejects_missing() {
        let args = json!({});
        let err = require_string_arg(&args, "plug_id").unwrap_err();
        assert_eq!(err.code, -32602);
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
