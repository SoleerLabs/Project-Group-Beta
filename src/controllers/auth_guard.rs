use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt, // Add this import
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,       
    pub role: String,   
    pub exp: usize,
}


#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
   
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header first
        if let Ok(TypedHeader(Authorization(bearer))) = parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            let token = bearer.token();
            
            let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
            let decoded = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_ref()),
                &Validation::default(),
            )
            .map_err(|e| {
                println!("JWT validation failed: {:?}", e);
                StatusCode::UNAUTHORIZED
            })?;
           
            return Ok(AuthUser {
    user_id: decoded.claims.sub,
    role: decoded.claims.role, 
});
        }

        // Fallback: try to get token from cookies
        let jar = axum_extra::extract::cookie::CookieJar::from_headers(&parts.headers);
        let token = jar
            .get("token")
            .map(|cookie| cookie.value().to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?;
       
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| {
            println!("JWT validation failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
       
       return Ok(AuthUser {
    user_id: decoded.claims.sub,
    role: decoded.claims.role, 
});
    }
}