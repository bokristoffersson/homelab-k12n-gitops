use axum::{middleware, routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::handlers::{health, settings, AppState};
use crate::auth::require_jwt_auth;

pub fn create_router(state: AppState) -> Router {
    // Protected settings routes (require JWT auth)
    let protected_routes = Router::new()
        .route("/api/v1/heatpump/settings", get(settings::get_all_settings))
        .route(
            "/api/v1/heatpump/settings/{device_id}/outbox",
            get(settings::get_outbox_entries_by_device),
        )
        .route(
            "/api/v1/heatpump/settings/{device_id}",
            get(settings::get_setting_by_device).patch(settings::update_setting),
        )
        .route(
            "/api/v1/heatpump/settings/outbox/{id}",
            get(settings::get_outbox_status),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_jwt_auth,
        ));

    // Public routes (health check)
    let public_routes = Router::new().route("/health", get(health::health_check));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
