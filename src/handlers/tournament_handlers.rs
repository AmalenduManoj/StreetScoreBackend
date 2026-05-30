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
            VALUES ($1, $2, $3::timestamp, $4::date, $5)
            RETURNING id",
    )
    .persistent(false)
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
        .persistent(false)
        .bind(team_id)
        .bind(tournament_id)
        .bind(claims.user_id)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().body(e.to_string());
        }
        // ensure a standing entry exists for this team with zeroed stats
        if let Err(e) = sqlx::query(
            "INSERT INTO tournament_standing (tournament_id, team_id, match_played, wons, losses, points, run_rate)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (tournament_id, team_id) DO NOTHING",
        )
        .persistent(false)
        .bind(tournament_id)
        .bind(team_id)
        .bind(0_i32)
        .bind(0_i32)
        .bind(0_i32)
        .bind(0_i32)
        .bind(0.0_f64)
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
        "SELECT id, name, location, start_date, end_date, created_by_user_id FROM tournaments ORDER BY start_date DESC, id DESC",
    )
    .persistent(false)
    .fetch_all(pool.get_ref())
    .await;

    match tournaments {
        Ok(tournaments) => HttpResponse::Ok().json(tournaments),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_my_tournaments(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let tournaments = sqlx::query_as::<_, Tournament>(
        "SELECT id, name, location, start_date, end_date, created_by_user_id
         FROM tournaments
         WHERE created_by_user_id = $1
         ORDER BY start_date DESC, id DESC",
    )
    .persistent(false)
    .bind(claims.user_id)
    .fetch_all(pool.get_ref())
    .await;

    match tournaments {
        Ok(tournaments) => HttpResponse::Ok().json(tournaments),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_tournament_by_id(pool: web::Data<PgPool>, id: web::Path<i64>) -> impl Responder {
    let tournament = sqlx::query_as::<_, Tournament>(
        "SELECT id, name, location, start_date, end_date, created_by_user_id FROM tournaments WHERE id = $1",
    )
    .persistent(false)
    .bind(id.into_inner())
    .fetch_optional(pool.get_ref())
    .await;

    match tournament {
        Ok(Some(tournament)) => HttpResponse::Ok().json(tournament),
        Ok(None) => HttpResponse::NotFound().body("Tournament not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
