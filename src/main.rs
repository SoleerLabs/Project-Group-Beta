//src/main.rs
use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::env;
use sqlx::postgres::PgPoolOptions;

pub mod models;
pub mod routers;
pub mod controllers;
pub mod app_state;

use app_state::AppState;
use routers::product::product_routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create Postgres pool");
    
    let app_state = AppState {
        db: std::sync::Arc::new(pool),
    };
    
    let app = Router::new()
        .route("/", get(root))
        .nest("/products", product_routes())
        .with_state(app_state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server is running on port: 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Server is live 🚀 on port: 3000"
}