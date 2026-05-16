use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use serde::Deserialize;
use crate::models::players::Players;
use crate::auth::jwt::Claims;

#[derive(Deserialize)]
pub struct CreatePlayerRequest {
    pub name: String,
    pub is_active: bool,
    pub dob: i32,
    pub role: String,
    pub profile_picture_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePlayerRequest {
    pub name: String,
    pub is_active: bool,
    pub dob: i32,
    pub role: String,
    pub profile_picture_url: Option<String>,
    pub bio: Option<String>,
}

pub async fn create_player(req: HttpRequest, pool: web::Data<PgPool>, data: web::Json<CreatePlayerRequest>) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM players WHERE user_id = $1)")
        .persistent(false)
        .bind(claims.user_id)
        .fetch_one(pool.get_ref())
        .await;

    match exists {
        Ok(true) => return HttpResponse::Conflict().body("User already has a player"),
        Ok(false) => (),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }

    let profile_pic = data.profile_picture_url.as_deref().unwrap_or("");
    let bio = data.bio.as_deref().unwrap_or("");

    let result = sqlx::query(
        "INSERT INTO players (name, is_active, dob, user_id, role, profile_picture_url, bio, team_id, runs_scored, wickets_taken, matches_played, batting_average, bowling_average, strike_rate, over_bowled, economy_rate, five_wicket_hauls, centuries, half_centuries, player_of_the_match_awards, player_of_the_series_awards, highest_score, best_bowling_figures, debut_date, last_match_date, ball_faced, fours, sixes, three_wicket_hauls, catches, stumpings) VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,0,0,0,0.0,0.0,0.0,0.0,0.0,0,0,0,0,0,0,''::text,NULL,NULL,0,0,0,0,0,0)"
    )
    .persistent(false)
    .bind(&data.name)
    .bind(data.is_active)
    .bind(data.dob)
    .bind(claims.user_id)
    .bind(&data.role)
    .bind(profile_pic)
    .bind(bio)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Player created successfully"})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_players(pool: web::Data<PgPool>) -> impl Responder {
    let players = sqlx::query_as::<_, Players>("SELECT * FROM players")
        .persistent(false)
        .fetch_all(pool.get_ref())
        .await;

    match players {
        Ok(players) => HttpResponse::Ok().json(players),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_player_by_id(pool: web::Data<PgPool>, id: web::Path<i64>) -> impl Responder {
    let player = sqlx::query_as::<_, Players>("SELECT * FROM players WHERE id = $1")
        .persistent(false)
        .bind(id.into_inner())
        .fetch_one(pool.get_ref())
        .await;

    match player {
        Ok(player) => HttpResponse::Ok().json(player),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_player(pool: web::Data<PgPool>, id: web::Path<i64>, data: web::Json<UpdatePlayerRequest>) -> impl Responder {
    let profile_pic = data.profile_picture_url.as_deref().unwrap_or("");
    let bio = data.bio.as_deref().unwrap_or("");

    let result = sqlx::query(
        "UPDATE players SET name=$1, is_active=$2, dob=$3, role=$4, profile_picture_url=$5, bio=$6 WHERE id=$7"
    )
    .persistent(false)
    .bind(&data.name)
    .bind(data.is_active)
    .bind(data.dob)
    .bind(&data.role)
    .bind(profile_pic)
    .bind(bio)
    .bind(id.into_inner())
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Player updated"})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn get_player_me(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().body("Missing auth claims"),
    };

    let player = sqlx::query_as::<_, Players>("SELECT * FROM players WHERE user_id = $1")
        .persistent(false)
        .bind(claims.user_id)
        .fetch_optional(pool.get_ref())
        .await;

    match player {
        Ok(Some(player)) => HttpResponse::Ok().json(player),
        Ok(None) => HttpResponse::NotFound().body("Player not found for this user"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
