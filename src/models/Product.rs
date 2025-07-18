//models/Product.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// The core Product structure representing items in the store.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock: i32,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload used when creating a new product via an API request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock: i32,
    pub category: Option<String>,
}

/// Payload used when updating a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub stock: Option<i32>,
    pub category: Option<String>,
}