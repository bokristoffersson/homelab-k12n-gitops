mod api;
mod auth;
mod config;
mod db;
mod error;
mod mcp;
mod repositories;

use config::Config;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    info!("Starting homelab-api");

    let cfg_path = std::env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.yaml".into());
    let cfg = Config::load(&cfg_path)?;
    info!("Configuration loaded");

    let pool = db::connect(&cfg.database.url).await?;
    sqlx::query("SELECT 1").execute(&pool).await?;
    info!("Connected to database");

    // Initialize JWT validator if auth is configured
    // Supports multi-issuer config (preferred) or legacy single-issuer config
    let jwt_validator = if let Some(ref auth_cfg) = cfg.auth {
        // Prefer multi-issuer configuration
        if !auth_cfg.issuers.is_empty() {
            match auth::JwtValidator::new_multi(auth_cfg.issuers.clone()).await {
                Ok(validator) => {
                    info!(
                        "JWT validator initialized with {} issuers: {:?}",
                        validator.issuer_count(),
                        auth_cfg
                            .issuers
                            .iter()
                            .map(|i| i.name.as_str())
                            .collect::<Vec<_>>()
                    );
                    Some(validator)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize multi-issuer JWT validator: {}", e);
                    None
                }
            }
        // Fall back to legacy single-issuer config
        } else if let (Some(jwks_url), Some(issuer)) = (&auth_cfg.jwks_url, &auth_cfg.issuer) {
            match auth::JwtValidator::new(jwks_url, issuer.clone()).await {
                Ok(validator) => {
                    info!("JWT validator initialized with JWKS from {}", jwks_url);
                    Some(validator)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize JWT validator: {}", e);
                    None
                }
            }
        } else {
            info!("No JWT issuer configuration found, skipping JWT validation");
            None
        }
    } else {
        None
    };

    let state = (pool.clone(), cfg.clone(), jwt_validator);
    let router = api::create_router(state);
    let addr = format!("{}:{}", cfg.api.host, cfg.api.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    info!("API server listening on {}", addr);

    // Set up graceful shutdown
    let serve = axum::serve(listener, router);
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received");
    };

    if let Err(e) = serve.with_graceful_shutdown(shutdown).await {
        tracing::error!(error = %e, "API server error");
    }

    info!("Application shutdown complete");
    Ok(())
}
