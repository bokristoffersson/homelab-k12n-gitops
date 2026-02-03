pub mod health;
pub mod settings;

use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    auth::JwtValidator,
    repositories::{OutboxRepository, SettingsRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<SettingsRepository>,
    pub outbox_repository: Arc<OutboxRepository>,
    pub pool: PgPool,
    pub jwt_validator: Option<JwtValidator>,
}
