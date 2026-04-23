use crate::api::handlers::{auth, energy, health, heatpump, temperature};
use crate::api::middleware::{require_jwt_auth, require_scope, RequiredScope};
use crate::auth::AppState;
use crate::mcp::handlers as mcp;
use axum::{extract::Request, middleware, routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::Level;

const SCOPE_READ_ENERGY: &str = "read:energy";
const SCOPE_READ_HEATPUMP: &str = "read:heatpump";
const SCOPE_READ_TEMPERATURE: &str = "read:temperature";

pub fn create_router(state: AppState) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new().route("/health", get(health::health));

    let energy_routes = Router::new()
        .route("/api/v1/energy/latest", get(energy::get_latest))
        .route("/api/v1/energy/hourly-total", get(energy::get_hourly_total))
        .route("/api/v1/energy/history", get(energy::get_history))
        .route(
            "/api/v1/energy/daily-summary",
            get(energy::get_daily_summary),
        )
        .route(
            "/api/v1/energy/monthly-summary",
            get(energy::get_monthly_summary),
        )
        .route(
            "/api/v1/energy/yearly-summary",
            get(energy::get_yearly_summary),
        )
        .layer(middleware::from_fn_with_state(
            RequiredScope(SCOPE_READ_ENERGY),
            require_scope,
        ));

    let heatpump_routes = Router::new()
        .route("/api/v1/heatpump/latest", get(heatpump::get_latest))
        .route(
            "/api/v1/heatpump/daily-summary",
            get(heatpump::get_daily_summary),
        )
        .layer(middleware::from_fn_with_state(
            RequiredScope(SCOPE_READ_HEATPUMP),
            require_scope,
        ));

    let temperature_routes = Router::new()
        .route("/api/v1/temperature/latest", get(temperature::get_latest))
        .route(
            "/api/v1/temperature/all-latest",
            get(temperature::get_all_latest),
        )
        .route("/api/v1/temperature/history", get(temperature::get_history))
        .layer(middleware::from_fn_with_state(
            RequiredScope(SCOPE_READ_TEMPERATURE),
            require_scope,
        ));

    let unscoped_user_routes = Router::new().route("/api/v1/user/info", get(auth::user_info));

    // All authenticated API routes: JWT auth is enforced once, scope checks layered per group above.
    let api_routes = Router::new()
        .merge(unscoped_user_routes)
        .merge(energy_routes)
        .merge(heatpump_routes)
        .merge(temperature_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_jwt_auth,
        ));

    // MCP routes (JWT auth via Bearer token)
    let mcp_routes = Router::new()
        .route("/mcp", get(mcp::sse_handler).post(mcp::rpc_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_jwt_auth,
        ));

    // Merge public and protected routes
    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .merge(mcp_routes)
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
                .on_request(|_request: &Request, _span: &tracing::Span| {
                    tracing::event!(Level::DEBUG, "received request");
                })
                .on_response(
                    |_response: &axum::response::Response,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::event!(Level::INFO, latency = ?latency, "request completed");
                    },
                )
                .on_failure(
                    |_error: tower_http::classify::ServerErrorsFailureClass,
                     _latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::event!(Level::ERROR, "request failed");
                    },
                ),
        )
}
