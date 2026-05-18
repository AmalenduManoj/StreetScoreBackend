use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Serialize,Deserialize,FromRow)]
pub struct progress{
    pub id: i64,
    pub match_id: i64,
    pub batter_id: i64,
    pub bowler_id: i64,
    pub runs_scored: i32,
    pub is_wicket: bool,
    pub over_number: i32,
    pub ball_number: i32,
    pub commentary: String,
    pub created_at: DateTime<Utc>
}