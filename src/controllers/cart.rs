use axum::{ extract::{ State, Path }, http::StatusCode, Json };
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use crate::{ app_state::AppState, models::CartItem };
use crate::controllers::auth_guard::AuthUser;

#[derive(Deserialize)]
pub struct UpdateCartItem {
    pub product_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn get_cart_items(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser
) -> Result<Json<Vec<CartItem>>, (StatusCode, Json<ErrorResponse>)> {
    let items = sqlx
        ::query_as::<_, CartItem>("SELECT * FROM cart_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&*state.db).await
        .map_err(|e| {
            eprintln!("DB fetch error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch cart items".to_string(),
                }),
            )
        })?;
    Ok(Json(items))
}

pub async fn add_cart_item(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(product_id): Path<Uuid>
) -> Result<Json<Vec<CartItem>>, (StatusCode, Json<ErrorResponse>)> {
    println!("User ID: {}, Product ID: {}", user_id, product_id);
    
    // First add/update the item in cart
    sqlx
        ::query(
            "INSERT INTO cart_items (user_id, product_id, quantity) VALUES ($1, $2, 1)
         ON CONFLICT (user_id, product_id) DO UPDATE SET quantity = cart_items.quantity + 1"
        )
        .bind(user_id)
        .bind(product_id)
        .execute(&*state.db).await
        .map_err(|e| {
            eprintln!("DB insert error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to add item to cart".to_string(),
                }),
            )
        })?;

    // Then fetch and return updated cart items
    let items = sqlx
        ::query_as::<_, CartItem>("SELECT * FROM cart_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&*state.db).await
        .map_err(|e| {
            eprintln!("DB fetch error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch updated cart items".to_string(),
                }),
            )
        })?;
   
    Ok(Json(items))
}

pub async fn remove_cart_item(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(product_id): Path<Uuid>
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx
        ::query(
            "UPDATE cart_items
         SET quantity = quantity - 1
         WHERE user_id = $1 AND product_id = $2 AND quantity > 1"
        )
        .bind(user_id)
        .bind(product_id)
        .execute(&*state.db).await
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update cart item".to_string(),
            })
        ))?;
        
    if result.rows_affected() == 0 {
        sqlx
            ::query("DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2")
            .bind(user_id)
            .bind(product_id)
            .execute(&*state.db).await
            .map_err(|_| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to remove cart item".to_string(),
                })
            ))?;
    }
    Ok(StatusCode::NO_CONTENT)
}