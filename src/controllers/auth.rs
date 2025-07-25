use crate::app_state::AppState;
use axum::{extract::{Json, State}, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::sync::Arc;
use chrono::{Utc, Duration};
use crate::models::User::User; 
use crate::controllers::auth_guard::AuthUser;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,      
    pub role: String,    
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


#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}


pub async fn dashboard(auth_user: AuthUser) -> impl IntoResponse {
    let message = format!("Welcome back, user ID: {} and you are a {}", auth_user.user_id, auth_user.role);
    (StatusCode::OK, message)
}

pub async fn register(
    State(state): State<Arc<AppState>>, 
    Json(payload): Json<RegisterRequest>, 
) -> Result<impl IntoResponse, impl IntoResponse> {
    println!("Register attempt for email: {}", payload.email);
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            println!("Password hashing error: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal error hashing password".to_string(),
                }),
            ));
        }
    };

    let role = payload.role.clone().unwrap_or_else(|| "customer".to_string());
    let user_id = Uuid::new_v4();

    let result = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, email, password_hash, role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, email, password_hash, role
        "#,
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&role)
    .fetch_one(&*state.db)
    .await;

    match result {
        Ok(user) => Ok((StatusCode::CREATED, Json(user))),
        Err(e) => {
            println!("Database error during registration: {:?}", e);
            if e.to_string().contains("duplicate key value") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: "Email already registered".to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Failed to register user".to_string(),
                    }),
                ))
            }
        }
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>, 
    Json(payload): Json<LoginRequest>, 
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
    .fetch_optional(&*state.db)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            println!("Database error during login: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error during login".to_string(),
                }),
            )
                .into_response();
        }
    };

    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Invalid password hash format".to_string(),
                }),
            )
                .into_response();
        }
    };

    let argon2 = Argon2::default();
    if let Err(_) = argon2.verify_password(payload.password.as_bytes(), &parsed_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        )
            .into_response();
    }

    let claims = Claims {
        sub: user.id,
        role: user.role.clone(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "JWT secret not set".to_string(),
                }),
            )
                .into_response();
        }
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Token generation failed".to_string(),
                }),
            )
                .into_response();
        }
    };

    let response = LoginResponse { token };
    (StatusCode::OK, Json(response)).into_response()
}
