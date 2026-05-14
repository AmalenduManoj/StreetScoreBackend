use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Serialize, Deserialize,FromRow)]
pub struct Tournament {
    pub id: i64,
    pub name: String,
    pub location: String,
    pub start_date: NaiveDateTime,
    pub end_date: chrono::NaiveDate,
}