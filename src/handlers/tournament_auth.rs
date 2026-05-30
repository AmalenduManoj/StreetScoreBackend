use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::auth::jwt::Claims;

pub fn claims_from_request(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

pub fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Missing auth claims"
    }))
}

pub fn forbidden(message: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(serde_json::json!({
        "error": message
    }))
}

pub async fn is_tournament_creator(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let creator: Option<i64> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM tournaments WHERE id = $1",
    )
    .persistent(false)
    .bind(tournament_id)
    .fetch_optional(pool)
    .await?;

    Ok(creator.map(|id| id == user_id).unwrap_or(false))
}

pub async fn is_tournament_creator_for_match(
    pool: &PgPool,
    match_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let creator: Option<i64> = sqlx::query_scalar(
        "SELECT t.created_by_user_id
         FROM matches m
         JOIN tournaments t ON t.id = m.tournament_id
         WHERE m.id = $1",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_optional(pool)
    .await?;

    Ok(creator.map(|id| id == user_id).unwrap_or(false))
}
