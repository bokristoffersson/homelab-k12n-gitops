use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::handlers::heatpump::{get_by_id, get_latest, health, list};
use crate::services::HeatpumpService;

pub fn create_router(service: HeatpumpService) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/heatpump", get(list))
        .route("/api/v1/heatpump/latest", get(get_latest))
        .route("/api/v1/heatpump/:ts", get(get_by_id))
        .layer(CorsLayer::permissive())
        .with_state(service)
}

