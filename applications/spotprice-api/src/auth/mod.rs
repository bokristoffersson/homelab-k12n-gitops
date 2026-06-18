pub mod jwt;

pub use jwt::JwtValidator;

/// The small slice of configuration the HTTP layer needs. Kept separate from the
/// full `Config` so the router state (cloned into every request) never carries
/// the database URL/credentials or the APNs key path.
#[derive(Clone)]
pub struct ApiContext {
    pub delivery_area: String,
    pub currency: String,
}

pub type AppState = (sqlx::PgPool, ApiContext, Option<JwtValidator>);
