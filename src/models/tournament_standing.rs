use serde::{Serialize,Deserialize};
use sqlx::{FromRow};

#[derive(Serialize,Deserialize,FromRow)]
pub struct tournament_standing{
    pub id: i64,
    pub tournament_id: i64,
    pub team_id: i64,
    pub match_played: i32,
    pub wons: i32,
    pub losses: i32,
    pub points: i32,
    pub run_rate: f64
}

#[derive(Serialize,Deserialize,FromRow)]
pub struct batsman_ranking{
    pub id: i64,
    pub tournament_id: i64,
    pub player_id: i64,
    pub runs: i32,
    pub ball_faced: i32,
    pub no_of_outs: i32
}

#[derive(Serialize,Deserialize,FromRow)]
pub struct bowler_ranking{
    pub id: i64,
    pub tournament_id: i64,
    pub player_id: i64,
    pub runs_given: i32,
    pub ball_bowled: i32,
    pub wickets: i32
}