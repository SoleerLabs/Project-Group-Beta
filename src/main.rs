use axum::{ routing::get, Router };

pub mod models;
pub mod routers;
pub mod controllers;

pub use routers::{ auth_routes, cart_routes, order_routes, product_routes };

#[tokio::main]
async fn main() {
    // our router
    let app = Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes())
        .nest("/cart", cart_routes())
        .nest("/orders", order_routes())
        .nest("/products", product_routes());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("Server is running on port: 3000");
}

// which calls one of these handlers
async fn root() -> &'static str {
    "Server is live 🚀"
}
