use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Match {
    pub id: i64,
    pub tournament_id: i64,
    pub team1_id: i64,
    pub team2_id: i64,
    pub venue: String,
    pub total_overs: i32,

    pub team1_score: i32,
    pub team1_wickets: i32,
    pub team1_overs: f32,

    pub team2_score: i32,
    pub team2_wickets: i32,
    pub team2_overs: f32,

    pub status: String,
}