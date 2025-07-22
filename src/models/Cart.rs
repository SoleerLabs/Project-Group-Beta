use serde::{ Deserialize, Serialize };
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, FromRow)]
pub struct CartItem {
    pub product_id: Uuid,
    pub quantity: i32,
    pub user_id: Uuid,
}
