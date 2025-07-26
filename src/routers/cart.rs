use axum::{ Router, routing::{ get, put, delete } };
use std::sync::Arc;

use crate::{
    controllers::cart::{ get_cart_items, add_cart_item, remove_cart_item },
    app_state::AppState,
};

pub fn cart_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_cart_items))
        .route("/add/:product_id", put(add_cart_item))
        .route("/remove/:product_id", delete(remove_cart_item))
}
