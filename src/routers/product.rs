use axum::{ routing::{ get }, Router };

use crate::controllers::product::*;

pub fn product_routes() -> Router {
    Router::new()
        .route("/", get(get_all_products).post(create_product))
        .route(
            "/{id}",
            get(get_product_by_id).put(update_product_by_id).delete(delete_product_by_id)
        )
}
