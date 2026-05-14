use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Team{
    pub id: i64,
    pub name: String,
    pub city: String,
    pub matches_played: i32,
    pub wins: i32,
    pub losses: i32,
    pub draws: i32
}