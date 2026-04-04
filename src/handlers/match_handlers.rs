use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use crate::models::r#match::Match;

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