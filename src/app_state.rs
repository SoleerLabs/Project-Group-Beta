//src/app_state.rs
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
}
