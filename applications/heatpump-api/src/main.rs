use heatpump_api::{create_pool, routes, Config};
use std::net::SocketAddr;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded");

    // Create database pool
    let pool = create_pool(&config).await?;
    info!("Database connection pool created");

    // Initialize repositories and services
    let repository = heatpump_api::repositories::HeatpumpRepository::new(pool);
    let service = heatpump_api::services::HeatpumpService::new(repository);

    // Create router
    let app = routes::create_router(service);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
