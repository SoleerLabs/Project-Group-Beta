use axum::{ routing::{ get }, Router };

use crate::controllers::order::*;

pub fn order_routes() -> Router {
    Router::new()
        .route("/", get(get_all_orders).post(create_order))
        .route("/{id}", get(get_order_by_id).put(update_order_by_id).delete(delete_order_by_id))
}
