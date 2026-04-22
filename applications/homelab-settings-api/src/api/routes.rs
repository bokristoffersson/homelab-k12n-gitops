use axum::{
    middleware,
    routing::{get, patch, post, put},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::handlers::{health, plugs, settings, AppState};
use crate::auth::{require_jwt_auth, require_scope};
use crate::mcp::handlers as mcp;

pub fn create_router(state: AppState) -> Router {
    let read_settings_routes = Router::new()
        .route("/api/v1/heatpump/settings", get(settings::get_all_settings))
        .route(
            "/api/v1/heatpump/settings/{device_id}",
            get(settings::get_setting_by_device),
        )
        .route(
            "/api/v1/heatpump/settings/{device_id}/outbox",
            get(settings::get_outbox_entries_by_device),
        )
        .route(
            "/api/v1/heatpump/settings/outbox/{id}",
            get(settings::get_outbox_status),
        )
        .route_layer(middleware::from_fn(|req, next| {
            require_scope("read:settings", req, next)
        }));

    let write_settings_routes = Router::new()
        .route(
            "/api/v1/heatpump/settings/{device_id}",
            patch(settings::update_setting),
        )
        .route_layer(middleware::from_fn(|req, next| {
            require_scope("write:settings", req, next)
        }));

    let read_plugs_routes = Router::new()
        .route("/api/v1/plugs", get(plugs::get_all_plugs))
        .route("/api/v1/plugs/{plug_id}", get(plugs::get_plug))
        .route(
            "/api/v1/plugs/{plug_id}/schedules",
            get(plugs::get_schedules),
        )
        .route(
            "/api/v1/plugs/{plug_id}/schedules/{schedule_id}",
            get(plugs::get_schedule),
        )
        .route_layer(middleware::from_fn(|req, next| {
            require_scope("read:plugs", req, next)
        }));

    let write_plugs_routes = Router::new()
        .route("/api/v1/plugs", post(plugs::create_plug))
        .route(
            "/api/v1/plugs/{plug_id}",
            put(plugs::update_plug)
                .patch(plugs::toggle_plug)
                .delete(plugs::delete_plug),
        )
        .route(
            "/api/v1/plugs/{plug_id}/schedules",
            post(plugs::create_schedule),
        )
        .route(
            "/api/v1/plugs/{plug_id}/schedules/{schedule_id}",
            put(plugs::update_schedule).delete(plugs::delete_schedule),
        )
        .route_layer(middleware::from_fn(|req, next| {
            require_scope("write:plugs", req, next)
        }));

    let protected_routes = Router::new()
        .merge(read_settings_routes)
        .merge(write_settings_routes)
        .merge(read_plugs_routes)
        .merge(write_plugs_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_jwt_auth,
        ));

    let mcp_routes = Router::new()
        .route("/mcp", get(mcp::sse_handler).post(mcp::rpc_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_jwt_auth,
        ));

    let public_routes = Router::new().route("/health", get(health::health_check));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(mcp_routes)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
