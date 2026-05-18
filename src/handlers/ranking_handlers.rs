use actix_web::{HttpResponse, web};
use sqlx::{PgPool, Row};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BatsmanRankingRequest {
    pub tournament_id: i64,
    pub player_id: i64,
    pub runs: i32,
    pub ball_faced: i32,
    pub no_of_outs: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BowlerRankingRequest {
    pub tournament_id: i64,
    pub player_id: i64,
    pub runs_given: i32,
    pub ball_bowled: i32,
    pub wickets: i32,
}

// ============== BATSMAN RANKINGS ==============

// Get all batsman rankings for a tournament
pub async fn get_batsman_rankings(
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query(
        "SELECT id, tournament_id, player_id, runs, ball_faced, no_of_outs
         FROM batsman_ranking
         WHERE tournament_id = $1
         ORDER BY runs DESC, ball_faced ASC"
    )
    .persistent(false)
    .bind(tournament_id.into_inner())
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let rankings: Vec<_> = rows.iter().enumerate().map(|(idx, row)| {
                let ball_faced: i32 = row.get("ball_faced");
                let strike_rate = if ball_faced > 0 {
                    (row.get::<i32, _>("runs") as f64 / ball_faced as f64) * 100.0
                } else {
                    0.0
                };
                
                serde_json::json!({
                    "rank": idx + 1,
                    "id": row.get::<i64, _>("id"),
                    "tournament_id": row.get::<i64, _>("tournament_id"),
                    "player_id": row.get::<i64, _>("player_id"),
                    "runs": row.get::<i32, _>("runs"),
                    "ball_faced": ball_faced,
                    "no_of_outs": row.get::<i32, _>("no_of_outs"),
                    "strike_rate": format!("{:.2}", strike_rate)
                })
            }).collect();
            HttpResponse::Ok().json(rankings)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get specific batsman ranking
pub async fn get_batsman_ranking(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let (tournament_id, player_id) = path.into_inner();
    
    let result = sqlx::query(
        "SELECT id, tournament_id, player_id, runs, ball_faced, no_of_outs
         FROM batsman_ranking
         WHERE tournament_id = $1 AND player_id = $2"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(player_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let ball_faced: i32 = row.get("ball_faced");
            let strike_rate = if ball_faced > 0 {
                (row.get::<i32, _>("runs") as f64 / ball_faced as f64) * 100.0
            } else {
                0.0
            };
            
            let ranking = serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "tournament_id": row.get::<i64, _>("tournament_id"),
                "player_id": row.get::<i64, _>("player_id"),
                "runs": row.get::<i32, _>("runs"),
                "ball_faced": ball_faced,
                "no_of_outs": row.get::<i32, _>("no_of_outs"),
                "strike_rate": format!("{:.2}", strike_rate)
            });
            HttpResponse::Ok().json(ranking)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Batsman ranking not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Create batsman ranking
pub async fn create_batsman_ranking(
    pool: web::Data<PgPool>,
    data: web::Json<BatsmanRankingRequest>,
) -> HttpResponse {
    let result = sqlx::query(
        "INSERT INTO batsman_ranking (tournament_id, player_id, runs, ball_faced, no_of_outs)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id"
    )
    .persistent(false)
    .bind(data.tournament_id)
    .bind(data.player_id)
    .bind(data.runs)
    .bind(data.ball_faced)
    .bind(data.no_of_outs)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let id: i64 = row.get("id");
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "message": "Batsman ranking created successfully"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Update batsman ranking
pub async fn update_batsman_ranking(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i64)>,
    data: web::Json<BatsmanRankingRequest>,
) -> HttpResponse {
    let (tournament_id, player_id) = path.into_inner();
    
    let result = sqlx::query(
        "UPDATE batsman_ranking 
         SET runs = $1, ball_faced = $2, no_of_outs = $3
         WHERE tournament_id = $4 AND player_id = $5"
    )
    .persistent(false)
    .bind(data.runs)
    .bind(data.ball_faced)
    .bind(data.no_of_outs)
    .bind(tournament_id)
    .bind(player_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Batsman ranking updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// ============== BOWLER RANKINGS ==============

// Get all bowler rankings for a tournament
pub async fn get_bowler_rankings(
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query(
        "SELECT id, tournament_id, player_id, runs_given, ball_bowled, wickets
         FROM bowler_ranking
         WHERE tournament_id = $1
         ORDER BY wickets DESC, runs_given ASC"
    )
    .persistent(false)
    .bind(tournament_id.into_inner())
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let rankings: Vec<_> = rows.iter().enumerate().map(|(idx, row)| {
                let ball_bowled: i32 = row.get("ball_bowled");
                let economy = if ball_bowled > 0 {
                    (row.get::<i32, _>("runs_given") as f64 / (ball_bowled as f64 / 6.0)) 
                } else {
                    0.0
                };
                
                serde_json::json!({
                    "rank": idx + 1,
                    "id": row.get::<i64, _>("id"),
                    "tournament_id": row.get::<i64, _>("tournament_id"),
                    "player_id": row.get::<i64, _>("player_id"),
                    "runs_given": row.get::<i32, _>("runs_given"),
                    "ball_bowled": ball_bowled,
                    "wickets": row.get::<i32, _>("wickets"),
                    "economy_rate": format!("{:.2}", economy)
                })
            }).collect();
            HttpResponse::Ok().json(rankings)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get specific bowler ranking
pub async fn get_bowler_ranking(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let (tournament_id, player_id) = path.into_inner();
    
    let result = sqlx::query(
        "SELECT id, tournament_id, player_id, runs_given, ball_bowled, wickets
         FROM bowler_ranking
         WHERE tournament_id = $1 AND player_id = $2"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(player_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let ball_bowled: i32 = row.get("ball_bowled");
            let economy = if ball_bowled > 0 {
                (row.get::<i32, _>("runs_given") as f64 / (ball_bowled as f64 / 6.0))
            } else {
                0.0
            };
            
            let ranking = serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "tournament_id": row.get::<i64, _>("tournament_id"),
                "player_id": row.get::<i64, _>("player_id"),
                "runs_given": row.get::<i32, _>("runs_given"),
                "ball_bowled": ball_bowled,
                "wickets": row.get::<i32, _>("wickets"),
                "economy_rate": format!("{:.2}", economy)
            });
            HttpResponse::Ok().json(ranking)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Bowler ranking not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Create bowler ranking
pub async fn create_bowler_ranking(
    pool: web::Data<PgPool>,
    data: web::Json<BowlerRankingRequest>,
) -> HttpResponse {
    let result = sqlx::query(
        "INSERT INTO bowler_ranking (tournament_id, player_id, runs_given, ball_bowled, wickets)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id"
    )
    .persistent(false)
    .bind(data.tournament_id)
    .bind(data.player_id)
    .bind(data.runs_given)
    .bind(data.ball_bowled)
    .bind(data.wickets)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let id: i64 = row.get("id");
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "message": "Bowler ranking created successfully"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Update bowler ranking
pub async fn update_bowler_ranking(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i64)>,
    data: web::Json<BowlerRankingRequest>,
) -> HttpResponse {
    let (tournament_id, player_id) = path.into_inner();
    
    let result = sqlx::query(
        "UPDATE bowler_ranking 
         SET runs_given = $1, ball_bowled = $2, wickets = $3
         WHERE tournament_id = $4 AND player_id = $5"
    )
    .persistent(false)
    .bind(data.runs_given)
    .bind(data.ball_bowled)
    .bind(data.wickets)
    .bind(tournament_id)
    .bind(player_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Bowler ranking updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
