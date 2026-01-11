use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use super::AppState;

pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    // Check database connectivity and table existence
    let health_result = state.repository.health_check().await;

    let mut response = json!({
        "status": "ok",
        "database": {
            "connected": false,
            "table_exists": false,
        }
    });

    match health_result {
        Ok((connected, table_exists)) => {
            response["database"]["connected"] = json!(connected);
            response["database"]["table_exists"] = json!(table_exists);
            
            if !table_exists {
                response["database"]["error"] = json!("Settings table does not exist. Please run migrations.");
            }
        }
        Err(e) => {
            response["database"]["error"] = json!(format!("Database error: {}", e));
        }
    }

    let status = if response["database"]["connected"].as_bool().unwrap_or(false) 
        && response["database"]["table_exists"].as_bool().unwrap_or(true) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}
