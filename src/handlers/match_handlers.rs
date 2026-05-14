use actix_web::{HttpResponse, Responder, dev::Path, web};
use sqlx::PgPool;
use crate::models::r#match::{self, Match};

pub async fn create_match(pool: web::Data<PgPool>, data: web::Json<Match>) -> impl Responder {
    let result = sqlx::query(
        "INSERT INTO matches (tournament_id, team1_id, team2_id, venue, total_overs, team1_score, team1_wickets, team1_overs, team2_score, team2_wickets, team2_overs, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id",
    )
    .bind(data.tournament_id)
    .bind(data.team1_id)
    .bind(data.team2_id)
    .bind(&data.venue)
    .bind(data.total_overs)
    .bind(data.team1_score)
    .bind(data.team1_wickets)
    .bind(data.team1_overs)
    .bind(data.team2_score)
    .bind(data.team2_wickets)
    .bind(data.team2_overs)
    .bind(&data.status)
    .persistent(false)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Match created"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_matches(pool: web::Data<PgPool>) -> impl Responder {
    let matches = sqlx::query_as::<_, Match>(
        "SELECT id, tournament_id, team1_id, team2_id, venue, total_overs, team1_score, team1_wickets, team1_overs::float4 AS team1_overs, team2_score, team2_wickets, team2_overs::float4 AS team2_overs, status FROM matches",
    )
    .persistent(false)
    .fetch_all(pool.get_ref())
    .await;

    match matches {
        Ok(matches) => HttpResponse::Ok().json(matches),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_live_match(pool:web::Data<PgPool>) -> impl Responder {
    let live_matchs = sqlx::query_as::<_, Match>(
        "SELECT id, tournament_id, team1_id, team2_id, venue, total_overs, team1_score, team1_wickets, team1_overs::float4 AS team1_overs, team2_score, team2_wickets, team2_overs::float4 AS team2_overs, status FROM matches WHERE status = 'live'",
    )
    .persistent(false)
    .fetch_all(pool.get_ref()) 
    .await;

    match live_matchs{
        Ok(matches) => HttpResponse::Ok().json(matches),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_match_by_id(pool:web::Data<PgPool>,id:web::Path<i64>) -> impl Responder{
    let required_match = sqlx::query_as::<_, Match>(
        "SELECT id, tournament_id, team1_id, team2_id, venue, total_overs, team1_score, team1_wickets, team1_overs::float4 AS team1_overs, team2_score, team2_wickets, team2_overs::float4 AS team2_overs, status FROM matches WHERE id = $1",
    )
    .bind(id.into_inner())
    .persistent(false)
    .fetch_one(pool.get_ref())
    .await;
    match required_match {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_match(pool:web::Data<PgPool>,id:web::Path<i64>,data:web::Json<Match>) -> impl Responder{
    let result = sqlx::query(
        "UPDATE matches SET tournament_id = $1, team1_id = $2, team2_id = $3, venue = $4, total_overs = $5, team1_score = $6, team1_wickets = $7, team1_overs = $8, team2_score = $9, team2_wickets = $10, team2_overs = $11, status = $12 WHERE id = $13",
    )
    .bind(data.tournament_id)
    .bind(data.team1_id)
    .bind(data.team2_id)
    .bind(&data.venue)
    .bind(data.total_overs)
    .bind(data.team1_score)
    .bind(data.team1_wickets)
    .bind(data.team1_overs)
    .bind(data.team2_score)
    .bind(data.team2_wickets)
    .bind(data.team2_overs)
    .bind(&data.status)
    .bind(id.into_inner())
    .persistent(false)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Match updated"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}