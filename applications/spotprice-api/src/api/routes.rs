use crate::api::handlers::{devices, health, prices};
use crate::api::middleware::{require_jwt_auth, require_scope, RequiredScope};
use crate::auth::AppState;
use axum::{
    extract::Request,
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing::Level;

const SCOPE_READ_SPOTPRICE: &str = "read:spotprice";

pub fn create_router(state: AppState) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new().route("/health", get(health::health));

    // All spotprice routes require the read:spotprice scope.
    let spotprice_routes = Router::new()
        .route("/api/v1/spotprice/today", get(prices::get_today))
        .route("/api/v1/spotprice/tomorrow", get(prices::get_tomorrow))
        .route("/api/v1/spotprice/latest", get(prices::get_latest))
        .route("/api/v1/spotprice/devices", post(devices::register))
        .route(
            "/api/v1/spotprice/devices/{token}",
            delete(devices::unregister),
        )
        .layer(middleware::from_fn_with_state(
            RequiredScope(SCOPE_READ_SPOTPRICE),
            require_scope,
        ));

    // JWT auth is enforced once for the whole API surface.
    let api_routes = Router::new()
        .merge(spotprice_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_jwt_auth,
        ));

    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_response(
                    |_response: &axum::response::Response,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::event!(Level::INFO, latency = ?latency, "request completed");
                    },
                ),
        )
}
