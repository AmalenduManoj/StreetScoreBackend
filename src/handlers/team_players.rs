use crate::auth::jwt::Claims;
use crate::models::players::Players;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TeamPlayerReq {
    pub team_id: i64,
    pub player_id: i64,
}

pub async fn add_player_to_team(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<TeamPlayerReq>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    // verify requester is the team creator
    let owner: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM teams WHERE id = $1",
    )
    .bind(data.team_id)
    .fetch_optional(pool.get_ref())
    .await;

    match owner {
        Ok(Some(owner_id)) => {
            if owner_id != claims.user_id {
                return HttpResponse::Forbidden().body("Only team creator can add players");
            }
        }
        Ok(None) => return HttpResponse::NotFound().body("Team not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    let result = sqlx::query(
        "INSERT INTO team_player_registry (team_id, player_id, user_id)
         VALUES ($1, $2, $3)
         ON CONFLICT (team_id, player_id) DO NOTHING",
    )
    .bind(data.team_id)
    .bind(data.player_id)
    .bind(claims.user_id)
    .persistent(false)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Player added to team"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_players_in_team(
    pool: web::Data<PgPool>,
    team_id: web::Path<i64>,
) -> impl Responder {
    let players = sqlx::query_as::<_, Players>(
        "SELECT p.* FROM team_player_registry tr JOIN players p ON tr.player_id = p.id WHERE tr.team_id = $1",
    )
    .bind(team_id.into_inner())
    .persistent(false)
    .fetch_all(pool.get_ref())
    .await;

    match players {
        Ok(players) => HttpResponse::Ok().json(players),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn remove_player_from_team(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<TeamPlayerReq>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    // verify requester is the team creator
    let owner: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM teams WHERE id = $1",
    )
    .bind(data.team_id)
    .fetch_optional(pool.get_ref())
    .await;

    match owner {
        Ok(Some(owner_id)) => {
            if owner_id != claims.user_id {
                return HttpResponse::Forbidden().body("Only team creator can remove players");
            }
        }
        Ok(None) => return HttpResponse::NotFound().body("Team not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    let result = sqlx::query("DELETE FROM team_player_registry WHERE team_id = $1 AND player_id = $2")
        .bind(data.team_id)
        .bind(data.player_id)
        .persistent(false)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Player removed from team"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_teams_for_player(
    pool: web::Data<PgPool>,
    player_id: web::Path<i64>,
) -> impl Responder {
    let teams = sqlx::query_scalar::<_, String>(
        "SELECT t.name
         FROM team_player_registry tp
         JOIN teams t ON t.id = tp.team_id
         WHERE tp.player_id = $1",
    )
    .bind(player_id.into_inner())
    .persistent(false)
    .fetch_all(pool.get_ref())
    .await;

    match teams {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_team_created_by_user(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let teams =
        sqlx::query_scalar::<_, String>("SELECT name FROM teams WHERE created_by_user_id = $1")
            .bind(claims.user_id)
            .persistent(false)
            .fetch_all(pool.get_ref())
            .await;

    match teams {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
