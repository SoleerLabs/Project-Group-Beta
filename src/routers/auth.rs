use axum::{ routing::post, Router };

use crate::controllers::auth::*;

pub fn auth_routes() -> Router {
    Router::new()
        .route("/sign-up", post(sign_up))
        .route("/sign-in", post(sign_in))
        .route("/sign-out", post(sign_out))
}
