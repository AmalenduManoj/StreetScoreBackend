use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}