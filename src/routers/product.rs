use axum::{Router, routing::{get, post, put, delete}};
use crate::controllers::product::*;
use crate::app_state::AppState;
use std::sync::Arc;

pub fn product_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all_products).post(create_product))
        .route("/{id}", get(get_product_by_id).put(update_product_by_id).delete(delete_product_by_id))
}