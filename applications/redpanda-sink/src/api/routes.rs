use crate::api::handlers::{auth, energy, health, heatpump};
use crate::api::middleware::require_auth;
use crate::config::Config;
use crate::db::DbPool;
use axum::{
    extract::Request,
    middleware::Next,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn create_router(pool: DbPool, config: Config) -> Router {
    let config_for_middleware = config.clone();

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/auth/login", post(auth::login));

    // Protected API routes (require authentication)
    let protected_routes = Router::new()
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
        .layer(axum::middleware::from_fn(
            move |mut request: Request, next: Next| {
                let config = config_for_middleware.clone();
                async move {
                    // Add config to extensions for middleware to access
                    request.extensions_mut().insert(config);
                    require_auth(request, next).await
                }
            },
        ));

    // Merge public and protected routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state((pool, config))
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
                .on_response(|_response: &axum::response::Response, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::event!(Level::INFO, latency = ?latency, "request completed");
                })
                .on_failure(|_error: tower_http::classify::ServerErrorsFailureClass, _latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::event!(Level::ERROR, "request failed");
                }),
        )
}
