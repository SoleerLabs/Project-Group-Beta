use axum::{Router, routing::{get, post}};
use std::sync::Arc;
use crate::controllers::auth::{register, login, dashboard};
use crate::app_state::AppState;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/dashboard", get(dashboard))
}