use crate::config::Config;
use crate::error::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(config: &Config) -> Result<DbPool> {
    let max_connections = config.database.max_connections.unwrap_or(10);
    
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&config.database.url)
        .await?;

    Ok(pool)
}

