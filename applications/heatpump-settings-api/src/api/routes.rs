use axum::{routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::handlers::{health, settings, AppState};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check (includes database status)
        .route("/health", get(health::health_check))
        // Settings API routes
        .route("/api/v1/heatpump/settings", get(settings::get_all_settings))
        // Outbox entries by device (must come before /{device_id} route)
        .route(
            "/api/v1/heatpump/settings/{device_id}/outbox",
            get(settings::get_outbox_entries_by_device),
        )
        .route(
            "/api/v1/heatpump/settings/{device_id}",
            get(settings::get_setting_by_device).patch(settings::update_setting),
        )
        // Outbox status by ID endpoint
        .route(
            "/api/v1/heatpump/settings/outbox/{id}",
            get(settings::get_outbox_status),
        )
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
