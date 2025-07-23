use axum::{ routing::get, Router };
use dotenvy::dotenv;
use std::env;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub mod models;
pub mod routers;
pub mod controllers;
pub mod app_state;

use app_state::AppState;
use routers::{ auth::auth_routes, cart::cart_routes, product::product_routes, order::order_routes};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await
        .expect("Failed to connect to database");

    // Create AppState and wrap the entire thing in Arc
    let state = Arc::new(AppState {
        db: Arc::new(pool),
    });

    let app = Router::new()
        .route("/", get(root))
        .nest("/auth", auth_routes())
        .nest("/cart", cart_routes())
        .nest("/products", product_routes())
        .nest("/orders", order_routes())
        .with_state(state); // Now passing Arc<AppState>

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server is running on port: 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Server is live 🚀"
}
