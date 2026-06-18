pub mod jwt;

pub use jwt::JwtValidator;
pub type AppState = (sqlx::PgPool, crate::config::Config, Option<JwtValidator>);
