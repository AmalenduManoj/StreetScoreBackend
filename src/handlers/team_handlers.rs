use actix_web::{HttpResponse, web,Responder};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

use crate::models::team::Team;

pub async fn create_team(pool: web::Data<PgPool>, data: web::Json<Team>) -> impl Responder {
    let result = sqlx::query(
        "INSERT INTO teams (name, city, matches_played, wins, losses, draws) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&data.name)
    .bind(&data.city)
    .bind(data.matches_played)
    .bind(data.wins)
    .bind(data.losses)
    .bind(data.draws)
    .persistent(false)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Team created"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_teams(pool: web::Data<PgPool>) -> impl Responder {
    let teams = sqlx::query_as::<_, Team>(
        "SELECT id, name, city, matches_played, wins, losses, draws FROM teams",
    )
    .persistent(false)
    .fetch_all(pool.get_ref())
    .await;

    match teams {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_team_by_id(pool: web::Data<PgPool>, id: web::Path<i64>) -> impl Responder {
    let team = sqlx::query_as::<_, Team>(
        "SELECT id, name, city, matches_played, wins, losses, draws FROM teams WHERE id = $1",
    )
    .bind(id.into_inner())
    .persistent(false)
    .fetch_one(pool.get_ref())
    .await;

    match team {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}