use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct TeamTournamentRegistry {
    pub id: i64,
    pub team_id: i64,
    pub tournament_id: i64,
    pub registered_at: chrono::NaiveDateTime,
    pub user_id: i64,
}