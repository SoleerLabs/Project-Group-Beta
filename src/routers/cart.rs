use axum::{ routing::{ delete, get, put }, Router };

use crate::controllers::cart::*;

pub fn cart_routes() -> Router {
    Router::new()
        .route("/", get(get_cart_items))
        .route("/add", put(add_cart_item))
        .route("/remove", delete(remove_cart_item))
}
