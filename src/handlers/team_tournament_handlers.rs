use actix_web::{HttpResponse, web,Responder,HttpRequest, HttpMessage};
use sqlx::PgPool;
use serde::Deserialize;
use crate::models::team::Team;
use crate::models::tournaments::Tournament;
use crate::auth::jwt::Claims;


#[derive(Deserialize)]
pub struct TeamTournamentRequest {
    pub team_ids: Vec<i64>,
}

pub async fn get_team_in_tournament(
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
) -> impl Responder {
    let tournament_exists: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "SELECT id FROM tournaments WHERE id = $1",
    )
    .persistent(false)
    .bind(*tournament_id)
    .fetch_optional(pool.get_ref())
    .await;

    match tournament_exists {
        Ok(Some(_)) => {}
        Ok(None) => return HttpResponse::NotFound().body("Tournament not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    let team_ids = match sqlx::query_as::<_, Team>(
        "SELECT t.id, t.name, t.city, t.matches_played, t.wins, t.losses, t.draws, t.created_by_user_id
         FROM team_tournament_registry tr
         JOIN teams t ON tr.team_id = t.id
         WHERE tr.tournament_id = $1
         ORDER BY t.name",
    )
    .persistent(false)
    .bind(*tournament_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(ids) => ids,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(team_ids)
}


pub async fn get_tournaments_for_team(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    team_id: web::Path<i64>,
) -> impl Responder {
    let _claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let tournament_ids = match sqlx::query_as::<_, Tournament>(
        "SELECT t.id, t.name, t.location, t.start_date, t.end_date, t.created_by_user_id
         FROM team_tournament_registry tr
         JOIN tournaments t ON tr.tournament_id = t.id
         WHERE tr.team_id = $1
         ORDER BY t.start_date DESC, t.id DESC",
    )
    .persistent(false)
    .bind(*team_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(ids) => ids,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(tournament_ids)
}

pub async fn add_team_to_tournament(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
    data: web::Json<TeamTournamentRequest>,
) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    // verify requester is tournament creator
    let creator_check: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM tournaments WHERE id = $1",
    )
    .persistent(false)
    .bind(*tournament_id)
    .fetch_optional(pool.get_ref())
    .await;

    match creator_check {
        Ok(Some(creator_id)) => {
            if creator_id != claims.user_id {
                return HttpResponse::Forbidden().body("Only tournament creator can add registrations");
            }
        }
        Ok(None) => return HttpResponse::NotFound().body("Tournament not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    for team_id in &data.team_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO team_tournament_registry (team_id, tournament_id, user_id)
             VALUES ($1, $2, $3)
             ON CONFLICT (team_id, tournament_id) DO NOTHING",
        )
        .persistent(false)
        .bind(team_id)
        .bind(*tournament_id)
        .bind(claims.user_id)
        .execute(pool.get_ref())
        .await
        {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
        // ensure a standing entry exists for this team in the tournament with zeroed fields
        if let Err(e) = sqlx::query(
            "INSERT INTO tournament_standing (tournament_id, team_id, match_played, wons, losses, points, run_rate)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (tournament_id, team_id) DO NOTHING",
        )
        .persistent(false)
        .bind(*tournament_id)
        .bind(team_id)
        .bind(0_i32)
        .bind(0_i32)
        .bind(0_i32)
        .bind(0_i32)
        .bind(0.0_f64)
        .execute(pool.get_ref())
        .await
        {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    }

    HttpResponse::Ok().body("Teams added to tournament successfully")
}

pub async fn delete_team_from_tournament(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    tournament_id: web::Path<i64>,
    data: web::Json<TeamTournamentRequest>,
) -> impl Responder {
    let _claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    // verify requester is tournament creator
    let creator_check: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM tournaments WHERE id = $1",
    )
    .persistent(false)
    .bind(*tournament_id)
    .fetch_optional(pool.get_ref())
    .await;

    match creator_check {
        Ok(Some(creator_id)) => {
            if creator_id != _claims.user_id {
                return HttpResponse::Forbidden().body("Only tournament creator can remove registrations");
            }
        }
        Ok(None) => return HttpResponse::NotFound().body("Tournament not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    for team_id in &data.team_ids {
        if let Err(e) = sqlx::query(
            "DELETE FROM team_tournament_registry WHERE team_id = $1 AND tournament_id = $2 AND user_id = $3",
        )
        .persistent(false)
        .bind(team_id)
        .bind(*tournament_id)
        .bind(_claims.user_id)
        .execute(pool.get_ref())
        .await
        {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    }

    HttpResponse::Ok().body("Teams removed from tournament successfully")
}
