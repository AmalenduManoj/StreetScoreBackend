use actix_web::{HttpResponse, web};
use sqlx::{PgPool, Row};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TournamentStandingRequest {
    pub tournament_id: i64,
    pub team_id: i64,
    pub match_played: i32,
    pub wons: i32,
    pub losses: i32,
    pub points: i32,
    pub run_rate: f64,
}

// Get all standings for a tournament
pub async fn get_tournament_standings(
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query(
        "SELECT id, tournament_id, team_id, match_played, wons, losses, points, run_rate
         FROM tournament_standing
         WHERE tournament_id = $1
         ORDER BY points DESC, run_rate DESC"
    )
    .persistent(false)
    .bind(tournament_id.into_inner())
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let standings: Vec<_> = rows.iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "tournament_id": row.get::<i64, _>("tournament_id"),
                    "team_id": row.get::<i64, _>("team_id"),
                    "match_played": row.get::<i32, _>("match_played"),
                    "wons": row.get::<i32, _>("wons"),
                    "losses": row.get::<i32, _>("losses"),
                    "points": row.get::<i32, _>("points"),
                    "run_rate": row.get::<f64, _>("run_rate")
                })
            }).collect();
            HttpResponse::Ok().json(standings)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get specific team standing
pub async fn get_team_standing(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let (tournament_id, team_id) = path.into_inner();
    
    let result = sqlx::query(
        "SELECT id, tournament_id, team_id, match_played, wons, losses, points, run_rate
         FROM tournament_standing
         WHERE tournament_id = $1 AND team_id = $2"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(team_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let standing = serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "tournament_id": row.get::<i64, _>("tournament_id"),
                "team_id": row.get::<i64, _>("team_id"),
                "match_played": row.get::<i32, _>("match_played"),
                "wons": row.get::<i32, _>("wons"),
                "losses": row.get::<i32, _>("losses"),
                "points": row.get::<i32, _>("points"),
                "run_rate": row.get::<f64, _>("run_rate")
            });
            HttpResponse::Ok().json(standing)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Standing not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Create or initialize standing
pub async fn create_standing(
    pool: web::Data<PgPool>,
    data: web::Json<TournamentStandingRequest>,
) -> HttpResponse {
    let result = sqlx::query(
        "INSERT INTO tournament_standing (tournament_id, team_id, match_played, wons, losses, points, run_rate)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id"
    )
    .persistent(false)
    .bind(data.tournament_id)
    .bind(data.team_id)
    .bind(data.match_played)
    .bind(data.wons)
    .bind(data.losses)
    .bind(data.points)
    .bind(data.run_rate)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let id: i64 = row.get("id");
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "message": "Standing created successfully"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Update standing
pub async fn update_standing(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i64)>,
    data: web::Json<TournamentStandingRequest>,
) -> HttpResponse {
    let (tournament_id, team_id) = path.into_inner();
    
    let result = sqlx::query(
        "UPDATE tournament_standing 
         SET match_played = $1, wons = $2, losses = $3, points = $4, run_rate = $5
         WHERE tournament_id = $6 AND team_id = $7"
    )
    .persistent(false)
    .bind(data.match_played)
    .bind(data.wons)
    .bind(data.losses)
    .bind(data.points)
    .bind(data.run_rate)
    .bind(tournament_id)
    .bind(team_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Standing updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get top teams (leaderboard)
pub async fn get_tournament_leaderboard(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, Option<i32>)>,
) -> HttpResponse {
    let (tournament_id, limit) = path.into_inner();
    let limit = limit.unwrap_or(10) as i64;
    
    let result = sqlx::query(
        "SELECT id, tournament_id, team_id, match_played, wons, losses, points, run_rate
         FROM tournament_standing
         WHERE tournament_id = $1
         ORDER BY points DESC, run_rate DESC
         LIMIT $2"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(limit)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let standings: Vec<_> = rows.iter().enumerate().map(|(idx, row)| {
                serde_json::json!({
                    "rank": idx + 1,
                    "id": row.get::<i64, _>("id"),
                    "tournament_id": row.get::<i64, _>("tournament_id"),
                    "team_id": row.get::<i64, _>("team_id"),
                    "match_played": row.get::<i32, _>("match_played"),
                    "wons": row.get::<i32, _>("wons"),
                    "losses": row.get::<i32, _>("losses"),
                    "points": row.get::<i32, _>("points"),
                    "run_rate": row.get::<f64, _>("run_rate")
                })
            }).collect();
            HttpResponse::Ok().json(standings)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
