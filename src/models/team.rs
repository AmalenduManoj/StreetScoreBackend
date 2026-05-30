use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub city: Option<String>,
    pub matches_played: i32,
    pub wins: i32,
    pub losses: i32,
    pub draws: i32,
    pub created_by_user_id: i64,
}

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub city: String,
    pub created_by_user_id: i64,
    pub player_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct UpdateTeamRequest {
    pub name: String,
    pub city: String,
}