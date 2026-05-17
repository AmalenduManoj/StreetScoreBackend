use actix_web::{HttpResponse, web,Responder,HttpRequest, HttpMessage};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::models::tournaments::Tournament;
use crate::auth::jwt::Claims;

#[derive(Deserialize)]
pub struct CreateTournamentRequest {
    name: String,
    location: String,
    start_date: String,
    end_date: String,
    pub team_ids: Vec<i64>,
}
#[derive(Serialize)]
pub struct CreateTournamentResponse {
    pub message: String,
    pub tournament_id: i64,
}

pub async fn create_tournament(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<CreateTournamentRequest>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let tournament_id = match sqlx::query_scalar::<_, i64>(
           "INSERT INTO tournaments (name, location, start_date, end_date, created_by_user_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id",
    )
    .bind(&data.name)
    .bind(&data.location)
    .bind(data.start_date.clone())
    .bind(data.end_date.clone())
        .bind(claims.user_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    for team_id in &data.team_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO team_tournament_registry (team_id, tournament_id, user_id)
             VALUES ($1, $2, $3)
             ON CONFLICT (team_id, tournament_id) DO NOTHING",
        )
        .bind(team_id)
        .bind(tournament_id)
        .bind(claims.user_id)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    HttpResponse::Ok().json(CreateTournamentResponse {
        message: "Tournament created".to_string(),
        tournament_id,
    })
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