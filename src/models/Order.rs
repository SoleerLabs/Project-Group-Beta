///models/Order.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

/// Main Order structure representing a customer's order
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total: BigDecimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Individual items within an order
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub vendor_id: Uuid,
    pub quantity: i32,
    pub price: BigDecimal,  // Price at time of purchase
}

/// Complete order details with items - used for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total: BigDecimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub items: Vec<OrderItemDetails>,
}

/// Order item with product information - used in OrderDetails
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemDetails {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,  // From products table
    pub vendor_id: Uuid,
    pub quantity: i32,
    pub price: BigDecimal,
    pub subtotal: BigDecimal,  // price * quantity
}

/// Payload for creating an order (minimal - cart conversion handles the data)
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    // For now, orders are created from cart, so no fields needed
    // Could add payment_method, shipping_address, etc. later
}

/// Payload for updating order status (vendor only)
#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatus {
    pub status: String,  // Should be "pending", "shipped", or "delivered"
}

/// Cart item structure for cart-to-order conversion
/// This matches your CartItem but includes product info for calculations
#[derive(Debug, FromRow)]
pub struct CartItemWithProduct {
    pub product_id: Uuid,
    pub quantity: i32,
    pub user_id: Uuid,
    pub product_name: String,
    pub price: BigDecimal,
    pub vendor_id: Uuid,
    pub stock: i32,
}

/// Response structure for order creation
#[derive(Debug, Serialize)]
pub struct OrderCreationResponse {
    pub order_id: Uuid,
    pub total: BigDecimal,
    pub status: String,
    pub message: String,
}

/// Simple order summary for listing orders
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderSummary {
    pub id: Uuid,
    pub total: BigDecimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub item_count: Option<i64>,  // Number of different products in order
}