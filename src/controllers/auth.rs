use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use jsonwebtoken::{encode, Header, EncodingKey};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::sync::Arc;
use chrono::{Utc, Duration};
use crate::models::User::User; 
use crate::controllers::auth_guard::AuthUser;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

pub async fn dashboard(auth_user: AuthUser) -> impl IntoResponse {
    let message = format!("Welcome back, user ID: {}", auth_user.user_id);
    (StatusCode::OK, message)
}

pub async fn register(
    State(state): State<Arc<AppState>>, 
    Json(payload): Json<RegisterRequest>, 
) -> Result<impl IntoResponse, StatusCode> { 
    println!("Register attempt for email: {}", payload.email);
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            println!("Password hashing error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();
    
    let role = payload.role.clone().unwrap_or_else(|| "customer".to_string());
    
    let result = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, password_hash, role
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&role)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("Database error during registration: {:?}", e);
        // Check if it's a unique constraint violation
        if e.to_string().contains("duplicate key value") {
            StatusCode::CONFLICT // 409 for duplicate user
        } else {
            StatusCode::BAD_REQUEST
        }
    })?;
    
    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn login(
    State(state): State<Arc<AppState>>, // Fixed: was State>
    Json(payload): Json<LoginRequest>, // Fixed: was Json
) -> impl IntoResponse {
    println!("Login attempt for email: {}", payload.email);
    
    let user = match sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, role
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(user)) => {
            println!("User found: {}", user.username);
            user
        },
        Ok(None) => {
            println!("No user found with email: {}", payload.email);
            return StatusCode::NOT_FOUND.into_response();
        },
        Err(e) => {
            println!("Database error during login: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        },
    };
    
    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            println!("Password hash parsing error: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        },
    };
    
    let argon2 = Argon2::default();
    if let Err(e) = argon2.verify_password(payload.password.as_bytes(), &parsed_hash) {
        println!("Password verification failed: {:?}", e);
        return StatusCode::UNAUTHORIZED.into_response();
    }
    
    println!("Password verified successfully");
    
    let claims = Claims {
        sub: user.id,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };
    
    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => {
            println!("JWT_SECRET found");
            secret
        },
        Err(e) => {
            println!("JWT_SECRET environment variable error: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        },
    };
    
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(token) => {
            println!("JWT token generated successfully");
            token
        },
        Err(e) => {
            println!("JWT encoding error: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        },
    };
    
    let response = LoginResponse { token };
    (StatusCode::OK, Json(response)).into_response()
}