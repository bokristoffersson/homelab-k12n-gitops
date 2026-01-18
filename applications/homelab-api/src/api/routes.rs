use crate::api::handlers::{auth, energy, health, heatpump, temperature};
use crate::auth::AppState;
use crate::mcp::handlers as mcp;
use axum::{extract::Request, routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn create_router(state: AppState) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new().route("/health", get(health::health));

    // API routes (authentication handled by oauth2-proxy/Authentik at ingress level)
    let api_routes = Router::new()
        .route("/mcp", get(mcp::sse_handler).post(mcp::rpc_handler))
        .route("/api/v1/user/info", get(auth::user_info))
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
        .route("/api/v1/heatpump/latest", get(heatpump::get_latest))
        .route(
            "/api/v1/heatpump/daily-summary",
            get(heatpump::get_daily_summary),
        )
        .route("/api/v1/temperature/latest", get(temperature::get_latest))
        .route(
            "/api/v1/temperature/all-latest",
            get(temperature::get_all_latest),
        )
        .route("/api/v1/temperature/history", get(temperature::get_history));

    // Merge public and API routes
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
