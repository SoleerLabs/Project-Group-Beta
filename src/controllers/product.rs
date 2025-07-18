//controllers/product.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use crate::{
    app_state::AppState,
    models::Product::{Product, CreateProduct, UpdateProduct},
};

pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProduct>,
) -> Result<(StatusCode, Json<Product>), StatusCode> {
    // Using a placeholder vendor_id. In real implementation, this would come from JWT token
    let placeholder_vendor_id = Uuid::new_v4();
    
    let query = sqlx::query_as!(
        Product,
        r#"
        INSERT INTO products (id, vendor_id, name, description, price, stock, category, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, vendor_id, name, description, price, stock, category, created_at, updated_at
        "#,
        Uuid::new_v4(),
        placeholder_vendor_id,
        payload.name,
        payload.description,
        payload.price,
        payload.stock,
        payload.category,
    );
    
    match query.fetch_one(&*state.db).await {
        Ok(product) => Ok((StatusCode::CREATED, Json(product))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_all_products(
    State(state): State<AppState>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    let query = sqlx::query_as!(
        Product,
        r#"
        SELECT id, vendor_id, name, description, price, stock, category, created_at, updated_at 
        FROM products
        ORDER BY name
        "#
    );
    
    match query.fetch_all(&*state.db).await {
        Ok(products) => Ok(Json(products)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_product_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Product>, StatusCode> {
    let query = sqlx::query_as!(
        Product,
        r#"
        SELECT id, vendor_id, name, description, price, stock, category, created_at, updated_at 
        FROM products
        WHERE id = $1
        "#,
        id
    );
    
    match query.fetch_optional(&*state.db).await {
        Ok(Some(product)) => Ok(Json(product)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_product_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProduct>,
) -> Result<Json<Product>, StatusCode> {
    let query = sqlx::query_as!(
        Product,
        r#"
        UPDATE products
        SET name = COALESCE($1, name), 
            description = COALESCE($2, description), 
            price = COALESCE($3, price), 
            stock = COALESCE($4, stock),
            category = COALESCE($5, category),
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, vendor_id, name, description, price, stock, category, created_at, updated_at
        "#,
        payload.name,
        payload.description,
        payload.price,
        payload.stock,
        payload.category,
        id
    );
    
    match query.fetch_optional(&*state.db).await {
        Ok(Some(product)) => Ok(Json(product)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_product_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let query = sqlx::query!(
        r#"
        DELETE FROM products
        WHERE id = $1
        "#,
        id
    );
    
    match query.execute(&*state.db).await {
        Ok(result) if result.rows_affected() > 0 => Ok(StatusCode::NO_CONTENT),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}