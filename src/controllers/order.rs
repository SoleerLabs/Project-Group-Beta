use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use crate::{
    app_state::AppState,
    controllers::auth_guard::AuthUser,
    models::Order::{
        Order, OrderItem, OrderDetails, OrderItemDetails, OrderSummary,
        CreateOrderRequest, UpdateOrderStatus, OrderCreationResponse,
        CartItemWithProduct,
    },
};

#[derive(Deserialize)]
pub struct OrderQueryParams {
    pub vendor: Option<bool>,  // ?vendor=true for vendor-specific orders
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// Get all orders - behavior depends on user role and query params
/// Customer: gets their own orders
/// Vendor with ?vendor=true: gets orders containing their products
pub async fn get_all_orders(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(params): Query<OrderQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    
    
    // Customer gets their own orders (or vendor gets personal orders)
    let orders = sqlx::query_as::<_, OrderSummary>(
        r#"
        SELECT o.id, o.total, o.status, o.created_at,
               COUNT(oi.id) as item_count
        FROM orders o
        LEFT JOIN order_items oi ON o.id = oi.order_id
        WHERE o.user_id = $1
        GROUP BY o.id, o.total, o.status, o.created_at
        ORDER BY o.created_at DESC
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        println!("Database error fetching customer orders: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch customer orders".into(),
            }),
        )
    })?;

    Ok(Json(orders).into_response())
}

/// Create order from current cart (customer only)
pub async fn create_order(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(_payload): Json<CreateOrderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    
    // Only customers can create orders
    if auth_user.role != "customer" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Only customers can create orders".into(),
            }),
        ));
    }

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        println!("Failed to start transaction: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to start transaction".into(),
            }),
        )
    })?;

    // Get cart items with product details
    let cart_items = sqlx::query_as::<_, CartItemWithProduct>(
        r#"
        SELECT ci.product_id, ci.quantity, ci.user_id,
               p.name as product_name, p.price, p.vendor_id, p.stock
        FROM cart_items ci
        JOIN products p ON ci.product_id = p.id
        WHERE ci.user_id = $1
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        println!("Failed to fetch cart items: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch cart items".into(),
            }),
        )
    })?;

    if cart_items.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Cart is empty. Add items to cart before creating an order".into(),
            }),
        ));
    }

    // Check stock availability and calculate total
    let mut total = BigDecimal::from(0);
    for item in &cart_items {
        if item.stock < item.quantity {
            println!("Insufficient stock for product {}: requested {}, available {}", 
                    item.product_id, item.quantity, item.stock);
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: format!("Insufficient stock for product {}. Requested: {}, Available: {}", 
                                 item.product_name, item.quantity, item.stock),
                }),
            ));
        }
        total += &item.price * BigDecimal::from(item.quantity);
    }

    // Create the order
    let order_id = Uuid::new_v4();
    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (id, user_id, total, status)
        VALUES ($1, $2, $3, 'pending')
        RETURNING id, user_id, total, status, created_at
        "#,
    )
    .bind(order_id)
    .bind(auth_user.user_id)
    .bind(&total)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        println!("Failed to create order: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create order".into(),
            }),
        )
    })?;

    // Create order items and update product stock
    for item in &cart_items {
        // Insert order item
        sqlx::query(
            r#"
            INSERT INTO order_items (order_id, product_id, vendor_id, quantity, price)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(order.id)
        .bind(item.product_id)
        .bind(item.vendor_id)
        .bind(item.quantity)
        .bind(&item.price)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            println!("Failed to create order item: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create order item".into(),
                }),
            )
        })?;

        // Update product stock
        sqlx::query(
            "UPDATE products SET stock = stock - $1 WHERE id = $2"
        )
        .bind(item.quantity)
        .bind(item.product_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            println!("Failed to update product stock: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update product stock".into(),
                }),
            )
        })?;
    }

    // Clear the cart
    sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
        .bind(auth_user.user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            println!("Failed to clear cart: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to clear cart".into(),
                }),
            )
        })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        println!("Failed to commit transaction: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to commit transaction".into(),
            }),
        )
    })?;

    // Simulate payment (always succeeds for now)
    println!("Payment simulated successfully for order {}", order.id);

    let response = OrderCreationResponse {
        order_id: order.id,
        total: order.total,
        status: order.status,
        message: "Order created successfully! Payment processed.".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// Get order by ID with full details
/// Customer: can only see their own orders
/// Vendor: can see orders containing their products
pub async fn get_order_by_id(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    
    // First get the order
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, user_id, total, status, created_at FROM orders WHERE id = $1"
    )
    .bind(order_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        println!("Database error fetching order: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch order".into(),
            }),
        )
    })?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Order not found".into(),
        }),
    ))?;

    // Check authorization
    let can_access = match auth_user.role.as_str() {
        "customer" => order.user_id == auth_user.user_id,
        "vendor" => {
            // Check if vendor has any products in this order
            let vendor_item_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM order_items WHERE order_id = $1 AND vendor_id = $2"
            )
            .bind(order_id)
            .bind(auth_user.user_id)
            .fetch_one(&*state.db)
            .await
            .map_err(|_| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to verify vendor access".into(),
                }),
            ))?;
            
            vendor_item_count > 0
        },
        _ => false,
    };

    if !can_access {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to view this order".into(),
            }),
        ));
    }

    // Get order items with product details
    let order_items = sqlx::query_as::<_, (Uuid, Uuid, String, Uuid, i32, BigDecimal)>(
        r#"
        SELECT oi.id, oi.product_id, p.name as product_name, 
               oi.vendor_id, oi.quantity, oi.price
        FROM order_items oi
        JOIN products p ON oi.product_id = p.id
        WHERE oi.order_id = $1
        ORDER BY oi.id
        "#,
    )
    .bind(order_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        println!("Database error fetching order items: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch order items".into(),
            }),
        )
    })?;

    let items: Vec<OrderItemDetails> = order_items
        .into_iter()
        .map(|(id, product_id, product_name, vendor_id, quantity, price)| {
            let subtotal = &price * BigDecimal::from(quantity);
            OrderItemDetails {
                id,
                product_id,
                product_name,
                vendor_id,
                quantity,
                price,
                subtotal,
            }
        })
        .collect();

    let order_details = OrderDetails {
        id: order.id,
        user_id: order.user_id,
        total: order.total,
        status: order.status,
        created_at: order.created_at,
        items,
    };

    Ok(Json(order_details).into_response())
}

/// Delete order by ID (admin only - could be restricted further)
pub async fn delete_order_by_id(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    
    // Only allow customers to delete their own orders (and only if pending)
    if auth_user.role != "customer" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Only customers can delete orders".into(),
            }),
        ));
    }

    // Check if order exists and belongs to user
    let order = sqlx::query_as::<_, Order>(
        "SELECT id, user_id, total, status, created_at FROM orders WHERE id = $1 AND user_id = $2"
    )
    .bind(order_id)
    .bind(auth_user.user_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Failed to fetch order".into(),
        }),
    ))?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Order not found or you don't have permission to delete it".into(),
        }),
    ))?;

    // Only allow deletion of pending orders
    if order.status != "pending" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Cannot delete order with status '{}'. Only pending orders can be deleted", order.status),
            }),
        ));
    }

    // Start transaction to restore stock and delete order
    let mut tx = state.db.begin().await.map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Failed to start transaction".into(),
        }),
    ))?;

    // Restore product stock
    sqlx::query(
        r#"
        UPDATE products 
        SET stock = stock + oi.quantity
        FROM order_items oi
        WHERE products.id = oi.product_id AND oi.order_id = $1
        "#,
    )
    .bind(order_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Failed to restore product stock".into(),
        }),
    ))?;

    // Delete order (order_items will cascade)
    sqlx::query("DELETE FROM orders WHERE id = $1")
        .bind(order_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to delete order".into(),
            }),
        ))?;

    tx.commit().await.map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Failed to commit transaction".into(),
        }),
    ))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Update order by ID (status updates for vendors)
pub async fn update_order_by_id(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatus>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    
    // Only vendors can update order status
    if auth_user.role != "vendor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Only vendors can update order status".into(),
            }),
        ));
    }

    // Validate status
    if !["pending", "shipped", "delivered"].contains(&payload.status.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid status '{}'. Valid statuses are: pending, shipped, delivered", payload.status),
            }),
        ));
    }

    // Check if vendor has products in this order
    let vendor_item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM order_items WHERE order_id = $1 AND vendor_id = $2"
    )
    .bind(order_id)
    .bind(auth_user.user_id)
    .fetch_one(&*state.db)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Failed to verify vendor access".into(),
        }),
    ))?;

    if vendor_item_count == 0 {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "You don't have permission to update this order. No products from your store in this order".into(),
            }),
        ));
    }

    // Update order status
    let updated_order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE orders 
        SET status = $1
        WHERE id = $2
        RETURNING id, user_id, total, status, created_at
        "#,
    )
    .bind(&payload.status)
    .bind(order_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Failed to update order status".into(),
        }),
    ))?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Order not found".into(),
        }),
    ))?;

    Ok(Json(updated_order).into_response())
}