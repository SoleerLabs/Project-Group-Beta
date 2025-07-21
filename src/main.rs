use axum::{ routing::get, Router };
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
pub mod models;
pub mod routers;
pub mod controllers;

pub use routers::{ auth_routes, cart_routes, order_routes, product_routes };

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    //Database Connection
     let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = Arc::new(controllers::auth::AppState { db: pool });
    // our router
    let app = Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes())
        .with_state(state.clone());

        // .nest("/cart", cart_routes())
        // .nest("/orders", order_routes())
        // .nest("/products", product_routes())

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        println!("Server is running on port: 3000");

    axum::serve(listener, app).await.unwrap();
    
}

// which calls one of these handlers
async fn root() -> &'static str {
    "Server is live 🚀"
}


