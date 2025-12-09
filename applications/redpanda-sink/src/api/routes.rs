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

pub fn create_router(pool: DbPool, config: Config) -> Router {
    let config_for_middleware = config.clone();

    Router::new()
        // Public routes
        .route("/api/v1/auth/login", post(auth::login))
        .route("/health", get(health::health))
        // Protected API routes - apply auth middleware to this group
        .route("/api/v1/energy/latest", get(energy::get_latest))
        .route("/api/v1/energy/hourly-total", get(energy::get_hourly_total))
        .route("/api/v1/energy/history", get(energy::get_history))
        .route("/api/v1/heatpump/latest", get(heatpump::get_latest))
        .layer(axum::middleware::from_fn(
            move |mut request: Request, next: Next| {
                let config = config_for_middleware.clone();
                async move {
                    // Add config to extensions for middleware to access
                    request.extensions_mut().insert(config);
                    require_auth(request, next).await
                }
            },
        ))
        .with_state((pool, config))
        .layer(tower_http::cors::CorsLayer::permissive())
}
