use actix_web::{HttpResponse, web,Responder};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::models::tournaments::Tournament;


pub async fn create_tournament(pool: web::Data<PgPool>, data: web::Json<Tournament>) -> impl Responder {
    let result = sqlx::query(
        "INSERT INTO tournaments (name, location, start_date, end_date) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&data.name)
    .bind(&data.location)
    .bind(data.start_date)
    .bind(data.end_date)
    .persistent(false)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Tournament created"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_tournaments(pool: web::Data<PgPool>) -> impl Responder {
    let tournaments = sqlx::query_as::<_, Tournament>(
        "SELECT id, name FROM tournaments",
    )
    .persistent(false)
    .fetch_all(pool.get_ref())
    .await;

    match tournaments {
        Ok(tournaments) => HttpResponse::Ok().json(tournaments),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_tournament_by_id(pool: web::Data<PgPool>, id: web::Path<i64>) -> impl Responder {
    let tournament = sqlx::query_as::<_, Tournament>(
        "SELECT id, name, location, start_date, end_date FROM tournaments WHERE id = $1",
    )
    .bind(id.into_inner())
    .persistent(false)
    .fetch_one(pool.get_ref())
    .await;

    match tournament {
        Ok(tournament) => HttpResponse::Ok().json(tournament),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}