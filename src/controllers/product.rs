use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use crate::{
    app_state::AppState,
    models::Product::{Product, CreateProduct, UpdateProduct},
};
use crate::controllers::auth_guard::AuthUser;


#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub search: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub category: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

//Added a middleWare to protect the routes from authorized access
pub async fn create_product(
    State(state): State<Arc<AppState>>,
       AuthUser { user_id, role }: AuthUser,//Check AuthGuard.rs in the controller folder to understand better
    Json(payload): Json<CreateProduct>,
) -> Result<(StatusCode, Json<Product>), StatusCode> {
    //Get role from authGuard and check if user is a vendor
     if role.to_lowercase() != "vendor" {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Use actual authenticated user ID
    let vendor_id = user_id; 

    
    let query = sqlx::query_as!(
        Product,
        r#"
        INSERT INTO products (id, vendor_id, name, description, price, stock, category, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, vendor_id, name, description, price, stock, category, created_at, updated_at
        "#,
        Uuid::new_v4(),
        vendor_id,
        payload.name,
        payload.description,
        payload.price,
        payload.stock,
        payload.category,
    );
    
  let result = query.fetch_one(&*state.db).await;
match result {
    Ok(product) => Ok((StatusCode::CREATED, Json(product))),
    Err(err) => {
        eprintln!("Error while creating product: {:?}", err);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }


}

}

pub async fn get_all_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProductQuery>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    let mut query_str = String::from("SELECT id, vendor_id, name, description, price, stock, category, created_at, updated_at FROM products WHERE 1=1");
    let mut bind_count = 0;
    
    // Build dynamic WHERE clause
    if params.search.is_some() {
        bind_count += 1;
        query_str.push_str(&format!(" AND (name ILIKE ${} OR description ILIKE ${})", bind_count, bind_count));
    }
    
    if params.min_price.is_some() {
        bind_count += 1;
        query_str.push_str(&format!(" AND price >= ${}", bind_count));
    }
    
    if params.max_price.is_some() {
        bind_count += 1;
        query_str.push_str(&format!(" AND price <= ${}", bind_count));
    }
    
    if params.category.is_some() {
        bind_count += 1;
        query_str.push_str(&format!(" AND category ILIKE ${}", bind_count));
    }
    
    query_str.push_str(" ORDER BY name");
    
    // Add pagination
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    bind_count += 1;
    query_str.push_str(&format!(" LIMIT ${}", bind_count));
    bind_count += 1;
    query_str.push_str(&format!(" OFFSET ${}", bind_count));
    
    // Build the query with dynamic binding
    let mut query = sqlx::query_as::<_, Product>(&query_str);
    
    if let Some(search) = &params.search {
        let search_term = format!("%{}%", search);
        query = query.bind(search_term);
    }
    
    if let Some(min_price) = params.min_price {
        query = query.bind(min_price);
    }
    
    if let Some(max_price) = params.max_price {
        query = query.bind(max_price);
    }
    
    if let Some(category) = &params.category {
        query = query.bind(category);
    }
    
    query = query.bind(limit).bind(offset);
    
    match query.fetch_all(&*state.db).await {
        Ok(products) => Ok(Json(products)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_product_by_id(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>, 
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
        AuthUser { user_id, role }: AuthUser,
    State(state): State<Arc<AppState>>, 
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
        AuthUser { user_id, role }: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,  
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