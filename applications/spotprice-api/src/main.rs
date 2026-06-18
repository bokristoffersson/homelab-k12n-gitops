mod api;
mod apns;
mod auth;
mod config;
mod db;
mod error;
mod fetcher;
mod nordpool;
mod repositories;

use config::Config;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,spotprice_api=debug"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();

    info!("Starting spotprice-api");

    let cfg_path = std::env::var("APP_CONFIG").unwrap_or_else(|_| "config/config.yaml".into());
    let cfg = Config::load(&cfg_path)?;
    info!("Configuration loaded");

    let pool = db::connect(&cfg.database.url).await?;
    sqlx::query("SELECT 1").execute(&pool).await?;
    info!("Connected to database");

    let jwt_validator = init_jwt_validator(&cfg).await?;

    // Build the APNs sender (push is skipped if not configured).
    let apns = match cfg.apns.as_ref() {
        Some(apns_cfg) => match apns::ApnsSender::from_config(apns_cfg) {
            Ok(sender) => {
                info!("APNs sender initialized (bundle {})", apns_cfg.bundle_id);
                Some(sender)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize APNs sender: {} (push disabled)", e);
                None
            }
        },
        None => {
            info!("APNs not configured, push disabled");
            None
        }
    };

    // Spawn the daily Nord Pool fetch scheduler.
    let fetcher = fetcher::Fetcher::new(pool.clone(), cfg.clone(), apns);
    let fetcher_handle = tokio::spawn(async move {
        fetcher.run().await;
    });

    // The HTTP layer only needs the delivery area/currency — not the DB URL or
    // APNs key path that the full Config carries.
    let api_ctx = auth::ApiContext {
        delivery_area: cfg.nordpool.delivery_area.clone(),
        currency: cfg.nordpool.currency.clone(),
    };
    let state = (pool.clone(), api_ctx, jwt_validator);
    let router = api::create_router(state);
    let addr = format!("{}:{}", cfg.api.host, cfg.api.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    info!("API server listening on {}", addr);

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

    fetcher_handle.abort();
    info!("Application shutdown complete");
    Ok(())
}

/// Initialize the multi-issuer JWT validator. Returns `None` only when no auth
/// is configured (dev/test). When issuers ARE configured, a JWKS fetch failure
/// is a hard startup error: the service must not boot into a state where it
/// cannot authenticate requests (which would otherwise 500 every protected
/// route until the pod is restarted, with no JWKS retry).
async fn init_jwt_validator(cfg: &Config) -> anyhow::Result<Option<auth::JwtValidator>> {
    let Some(auth_cfg) = cfg.auth.as_ref() else {
        info!("No auth configured, JWT validation disabled");
        return Ok(None);
    };
    if auth_cfg.issuers.is_empty() {
        info!("No JWT issuers configured, JWT validation disabled");
        return Ok(None);
    }
    let validator = auth::JwtValidator::new_multi(auth_cfg.issuers.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize JWT validator: {}", e))?;
    info!(
        "JWT validator initialized with {} issuer(s)",
        validator.issuer_count()
    );
    Ok(Some(validator))
}
