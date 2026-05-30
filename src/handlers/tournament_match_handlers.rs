use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use serde::{Serialize, Deserialize};

use crate::handlers::tournament_auth::{claims_from_request, forbidden, is_tournament_creator, unauthorized};

fn tournament_match_response(row: &PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.get::<i64, _>("id"),
        "tournament_id": row.get::<i64, _>("tournament_id"),
        "match_id": row.get::<i64, _>("match_id"),
        "match_number": row.get::<i32, _>("match_number"),
        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "match_details": {
            "id": row.get::<i64, _>("match_record_id"),
            "tournament_id": row.get::<i64, _>("match_tournament_id"),
            "team1_id": row.get::<i64, _>("team1_id"),
            "team2_id": row.get::<i64, _>("team2_id"),
            "venue": row.get::<String, _>("venue"),
            "total_overs": row.get::<i32, _>("total_overs"),
            "team1_score": row.get::<i32, _>("team1_score"),
            "team1_wickets": row.get::<i32, _>("team1_wickets"),
            "team1_overs": row.get::<f64, _>("team1_overs"),
            "team2_score": row.get::<i32, _>("team2_score"),
            "team2_wickets": row.get::<i32, _>("team2_wickets"),
            "team2_overs": row.get::<f64, _>("team2_overs"),
            "status": row.get::<String, _>("status")
        }
    })
}

#[derive(Serialize, Deserialize)]
pub struct MatchCreateRequest {
    pub team1_id: i64,
    pub team2_id: i64,
    pub venue: String,
    pub total_overs: i32,
    pub team1_score: i32,
    pub team1_wickets: i32,
    pub team1_overs: f64,
    pub team2_score: i32,
    pub team2_wickets: i32,
    pub team2_overs: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct TournamentMatchCreateRequest {
    pub match_number: i32,
    pub match_data: MatchCreateRequest,
}

#[derive(Serialize, Deserialize)]
pub struct TournamentMatchUpdateRequest {
    pub tournament_id: i64,
    pub match_id: i64,
    pub match_number: i32,
}

// Create a tournament match entry
pub async fn create_tournament_match(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    tournament_id: web::Path<i64>,
    data: web::Json<TournamentMatchCreateRequest>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Some(claims) => claims,
        None => return unauthorized(),
    };

    let tournament_id = tournament_id.into_inner();

    let allowed = match is_tournament_creator(pool.get_ref(), tournament_id, claims.user_id).await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if !allowed {
        return forbidden("Only the tournament creator can create matches");
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    };

    let result = sqlx::query(
        "INSERT INTO matches (
            tournament_id, team1_id, team2_id, venue, total_overs,
            team1_score, team1_wickets, team1_overs,
            team2_score, team2_wickets, team2_overs, status
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         RETURNING id"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(data.match_data.team1_id)
    .bind(data.match_data.team2_id)
    .bind(&data.match_data.venue)
    .bind(data.match_data.total_overs)
    .bind(data.match_data.team1_score)
    .bind(data.match_data.team1_wickets)
    .bind(data.match_data.team1_overs)
    .bind(data.match_data.team2_score)
    .bind(data.match_data.team2_wickets)
    .bind(data.match_data.team2_overs)
    .bind(&data.match_data.status)
    .fetch_one(&mut *tx)
    .await;

    let match_id = match result {
        Ok(row) => row.get::<i64, _>("id"),
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    };

    let result = sqlx::query(
        "INSERT INTO tournament_match (tournament_id, match_id, match_number, created_at)
         VALUES ($1, $2, $3, NOW())
         RETURNING id"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(match_id)
    .bind(data.match_number)
    .fetch_one(&mut *tx)
    .await;

    match result {
        Ok(row) => {
            let id: i64 = row.get("id");
            match tx.commit().await {
                Ok(_) => HttpResponse::Created().json(serde_json::json!({
                    "id": id,
                    "match_id": match_id,
                    "message": "Tournament match created successfully"
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get all matches in a tournament
pub async fn get_tournament_matches(
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query(
        "SELECT tm.id, tm.tournament_id, tm.match_id, tm.match_number, tm.created_at,
                m.id AS match_record_id, m.tournament_id AS match_tournament_id,
                m.team1_id, m.team2_id, m.venue, m.total_overs,
                m.team1_score, m.team1_wickets, m.team1_overs,
                m.team2_score, m.team2_wickets, m.team2_overs, m.status
         FROM tournament_match tm
         JOIN matches m ON tm.match_id = m.id
         WHERE tm.tournament_id = $1
         ORDER BY match_number ASC"
    )
    .persistent(false)
    .bind(tournament_id.into_inner())
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let matches: Vec<_> = rows.iter().map(tournament_match_response).collect();
            HttpResponse::Ok().json(matches)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get specific match in tournament
pub async fn get_tournament_match(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i32)>,
) -> HttpResponse {
    let (tournament_id, match_number) = path.into_inner();
    
    let result = sqlx::query(
        "SELECT tm.id, tm.tournament_id, tm.match_id, tm.match_number, tm.created_at,
            m.id AS match_record_id, m.tournament_id AS match_tournament_id,
            m.team1_id, m.team2_id, m.venue, m.total_overs,
            m.team1_score, m.team1_wickets, m.team1_overs,
            m.team2_score, m.team2_wickets, m.team2_overs, m.status
         FROM tournament_match tm
         JOIN matches m ON tm.match_id = m.id
         WHERE tm.tournament_id = $1 AND tm.match_number = $2"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(match_number)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            HttpResponse::Ok().json(tournament_match_response(&row))
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Tournament match not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get match by ID
pub async fn get_match_by_id(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query(
        "SELECT tm.id, tm.tournament_id, tm.match_id, tm.match_number, tm.created_at,
            m.id AS match_record_id, m.tournament_id AS match_tournament_id,
            m.team1_id, m.team2_id, m.venue, m.total_overs,
            m.team1_score, m.team1_wickets, m.team1_overs,
            m.team2_score, m.team2_wickets, m.team2_overs, m.status
         FROM tournament_match tm
         JOIN matches m ON tm.match_id = m.id
         WHERE tm.id = $1"
    )
    .persistent(false)
    .bind(id.into_inner())
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            HttpResponse::Ok().json(tournament_match_response(&row))
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Tournament match not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Update tournament match
pub async fn update_tournament_match(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    data: web::Json<TournamentMatchUpdateRequest>,
) -> HttpResponse {
    let id = path.into_inner();
    let tournament_id = data.tournament_id;
    let match_id = data.match_id;
    
    let result = sqlx::query(
        "UPDATE tournament_match 
         SET tournament_id = $1, match_id = $2, match_number = $3
         WHERE id = $4"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(match_id)
    .bind(data.match_number)
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Tournament match updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Delete tournament match
pub async fn delete_tournament_match(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query("DELETE FROM tournament_match WHERE id = $1")
        .persistent(false)
        .bind(id.into_inner())
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Tournament match deleted successfully"
        })),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Tournament match not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// Get match details with full information
pub async fn get_match_full_details(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i32)>,
) -> HttpResponse {
    let (tournament_id, match_number) = path.into_inner();
    
    let result = sqlx::query(
        "SELECT tm.id, tm.tournament_id, tm.match_id, tm.match_number, tm.created_at,
                 m.id AS match_record_id, m.tournament_id AS match_tournament_id,
                 m.team1_id, m.team2_id, m.venue, m.total_overs, m.status,
                 m.team1_score, m.team2_score, m.team1_wickets, m.team2_wickets,
                 m.team1_overs, m.team2_overs
         FROM tournament_match tm
         JOIN matches m ON tm.match_id = m.id
            WHERE tm.tournament_id = $1 AND tm.match_number = $2"
    )
    .persistent(false)
    .bind(tournament_id)
    .bind(match_number)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let details = serde_json::json!({
                "tournament_match_id": row.get::<i64, _>("id"),
                "tournament_id": row.get::<i64, _>("tournament_id"),
                "match_id": row.get::<i64, _>("match_id"),
                "match_number": row.get::<i32, _>("match_number"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "match_details": {
                    "team1_id": row.get::<i64, _>("team1_id"),
                    "team2_id": row.get::<i64, _>("team2_id"),
                    "venue": row.get::<String, _>("venue"),
                    "status": row.get::<String, _>("status"),
                    "team1_score": row.get::<i32, _>("team1_score"),
                    "team2_score": row.get::<i32, _>("team2_score"),
                    "team1_wickets": row.get::<i32, _>("team1_wickets"),
                    "team2_wickets": row.get::<i32, _>("team2_wickets")
                }
            });
            HttpResponse::Ok().json(details)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Match not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
