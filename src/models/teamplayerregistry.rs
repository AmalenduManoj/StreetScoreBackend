use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct teamplayerregistry {
    pub id: i64,
    pub team_id: i64,
    pub player_id: i64,
    pub user_id: i64,
}