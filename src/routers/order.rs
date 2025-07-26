use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use crate::controllers::order::*;
use crate::app_state::AppState;
use std::sync::Arc;

pub fn order_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Customer order routes
        .route("/", get(get_all_orders).post(create_order))
        .route("/:id", get(get_order_by_id).put(update_order_by_id).delete(delete_order_by_id))

}