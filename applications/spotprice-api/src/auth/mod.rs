pub mod jwt;

pub use jwt::JwtValidator;
pub type AppState = (sqlx::PgPool, crate::config::Config, Option<JwtValidator>);

#[allow(unused_imports)]
pub use jwt::{create_token, validate_token};
