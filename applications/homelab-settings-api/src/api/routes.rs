use axum::{middleware, routing::get, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::handlers::{health, plugs, settings, AppState};
use crate::auth::require_jwt_auth;

pub fn create_router(state: AppState) -> Router {
    // Protected settings routes (require JWT auth)
    let protected_settings_routes = Router::new()
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
        );

    // Protected plug routes (require JWT auth)
    let protected_plug_routes = Router::new()
        .route(
            "/api/v1/plugs",
            get(plugs::get_all_plugs).post(plugs::create_plug),
        )
        .route(
            "/api/v1/plugs/{plug_id}",
            get(plugs::get_plug)
                .put(plugs::update_plug)
                .patch(plugs::toggle_plug)
                .delete(plugs::delete_plug),
        )
        .route(
            "/api/v1/plugs/{plug_id}/schedules",
            get(plugs::get_schedules).post(plugs::create_schedule),
        )
        .route(
            "/api/v1/plugs/{plug_id}/schedules/{schedule_id}",
            get(plugs::get_schedule)
                .put(plugs::update_schedule)
                .delete(plugs::delete_schedule),
        );

    // Combine protected routes with auth middleware
    let protected_routes = Router::new()
        .merge(protected_settings_routes)
        .merge(protected_plug_routes)
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
