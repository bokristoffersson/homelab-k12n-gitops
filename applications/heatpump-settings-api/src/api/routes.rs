use axum::{
    routing::{get, patch},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::handlers::{health, settings, AppState};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check (no auth)
        .route("/health", get(health::health_check))
        // Settings API routes
        .route("/api/v1/heatpump/settings", get(settings::get_all_settings))
        .route(
            "/api/v1/heatpump/settings/:device_id",
            get(settings::get_setting_by_device).patch(settings::update_setting),
        )
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
