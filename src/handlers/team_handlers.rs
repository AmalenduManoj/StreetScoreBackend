use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use sqlx::PgPool;

use crate::models::team::{CreateTeamRequest, Team};

pub async fn create_team(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    data: web::Json<CreateTeamRequest>,
) -> impl Responder {
    let claims = match req.extensions().get::<crate::auth::jwt::Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let team_id = match sqlx::query_scalar::<_, i64>(
        "INSERT INTO teams (name, city, matches_played, wins, losses, draws, created_by_user_id)
         VALUES ($1, $2, 0, 0, 0, 0, $3)
         RETURNING id",
    )
    .persistent(false)
    .bind(&data.name)
    .bind(&data.city)
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

    for player_id in &data.player_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO team_player_registry (team_id, player_id, user_id)
             VALUES ($1, $2, $3)
             ON CONFLICT (team_id, player_id) DO NOTHING",
        )
        .persistent(false)
        .bind(team_id)
        .bind(player_id)
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

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Team created",
        "team_id": team_id
    }))
}

pub async fn get_teams(pool: web::Data<PgPool>) -> impl Responder {
    let teams = sqlx::query_as::<_, Team>(
        "SELECT id, name, city, matches_played, wins, losses, draws, created_by_user_id FROM teams",
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
        "SELECT id, name, city, matches_played, wins, losses, draws, created_by_user_id FROM teams WHERE id = $1",
    )
    .bind(id.into_inner())
    .persistent(false)
    .fetch_optional(pool.get_ref())
    .await;

    match team {
        Ok(Some(team)) => HttpResponse::Ok().json(team),
        Ok(None) => HttpResponse::NotFound().body("Team not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}