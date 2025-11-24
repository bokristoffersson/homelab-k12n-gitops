pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

pub use config::Config;
pub use db::create_pool;
pub use error::{AppError, Result};

