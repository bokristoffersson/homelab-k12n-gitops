pub mod health;
pub mod plugs;
pub mod settings;

use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    auth::JwtValidator,
    repositories::{OutboxRepository, PlugsRepository, SchedulesRepository, SettingsRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<SettingsRepository>,
    pub outbox_repository: Arc<OutboxRepository>,
    pub plugs_repository: Arc<PlugsRepository>,
    pub schedules_repository: Arc<SchedulesRepository>,
    pub pool: PgPool,
    pub jwt_validator: Option<JwtValidator>,
}
