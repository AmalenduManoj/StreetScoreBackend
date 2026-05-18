use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, FromRow)]
pub struct tournament_match{
    pub id: i64,
    pub tournament_id: i64,
    pub match_id: i64,
    pub match_number: i32,
    pub created_at: DateTime<Utc>
}

