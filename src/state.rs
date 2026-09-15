use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;

pub struct AppStateInner {
    pub db: PgPool,
    pub config: Config,
}

pub type AppState = Arc<AppStateInner>;
