use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use sqlx::{PgPool, Row};
use crate::models::progress::progress;
use chrono::Utc;

use crate::handlers::match_lineup_handlers::{after_progress_recorded, validate_progress_players};
use crate::handlers::match_ops::load_match;
use crate::handlers::tournament_auth::{claims_from_request, forbidden, is_tournament_creator_for_match, unauthorized};

pub async fn create_progress(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    data: web::Json<progress>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Some(claims) => claims,
        None => return unauthorized(),
    };

    let allowed = match is_tournament_creator_for_match(pool.get_ref(), data.match_id, claims.user_id).await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if !allowed {
        return forbidden("Only the tournament creator can record match progress");
    }

    let m = match load_match(pool.get_ref(), data.match_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({ "error": "Match not found" }));
        }
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if m.status != "live" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Progress can only be added while the match is live"
        }));
    }

    if let Err(response) = validate_progress_players(pool.get_ref(), data.match_id, data.batter_id, data.bowler_id).await {
        return response;
    }

    let result = sqlx::query(
        "INSERT INTO progress (match_id, batter_id, bowler_id, runs_scored, is_wicket, over_number, ball_number, commentary, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id"
    )
    .persistent(false)
    .bind(data.match_id)
    .bind(data.batter_id)
    .bind(data.bowler_id)
    .bind(data.runs_scored)
    .bind(data.is_wicket)
    .bind(data.over_number)
    .bind(data.ball_number)
    .bind(&data.commentary)
    .bind(data.created_at)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let id: i64 = row.get("id");
            if let Err(e) = after_progress_recorded(pool.get_ref(), data.match_id).await {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                }));
            }
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "message": "Delivery recorded successfully"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_progress_by_match(
    pool: web::Data<PgPool>,
    match_id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query_as::<_, progress>(
        "SELECT id, match_id, batter_id, bowler_id, runs_scored, is_wicket, over_number, ball_number, commentary, created_at
         FROM progress
         WHERE match_id = $1
         ORDER BY over_number, ball_number"
    )
    .persistent(false)
    .bind(match_id.into_inner())
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(deliveries) => HttpResponse::Ok().json(deliveries),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_progress_by_over(
    pool: web::Data<PgPool>,
    path: web::Path<(i64, i32)>,
) -> HttpResponse {
    let (match_id, over_number) = path.into_inner();
    
    let result = sqlx::query_as::<_, progress>(
        "SELECT id, match_id, batter_id, bowler_id, runs_scored, is_wicket, over_number, ball_number, commentary, created_at
         FROM progress
         WHERE match_id = $1 AND over_number = $2
         ORDER BY ball_number"
    )
    .persistent(false)
    .bind(match_id)
    .bind(over_number)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(deliveries) => HttpResponse::Ok().json(deliveries),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_progress_by_id(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query_as::<_, progress>(
        "SELECT id, match_id, batter_id, bowler_id, runs_scored, is_wicket, over_number, ball_number, commentary, created_at
         FROM progress
         WHERE id = $1"
    )
    .persistent(false)
    .bind(id.into_inner())
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(delivery) => HttpResponse::Ok().json(delivery),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Delivery not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn update_progress(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    data: web::Json<progress>,
) -> HttpResponse {
    let id = path.into_inner();
    
    let result = sqlx::query(
        "UPDATE progress 
         SET batter_id = $1, bowler_id = $2, runs_scored = $3, is_wicket = $4, 
             over_number = $5, ball_number = $6, commentary = $7
         WHERE id = $8"
    )
    .persistent(false)
    .bind(data.batter_id)
    .bind(data.bowler_id)
    .bind(data.runs_scored)
    .bind(data.is_wicket)
    .bind(data.over_number)
    .bind(data.ball_number)
    .bind(&data.commentary)
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Delivery updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn delete_progress(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query("DELETE FROM progress WHERE id = $1")
        .persistent(false)
        .bind(id.into_inner())
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok().json(serde_json::json!({
            "message": "Delivery deleted successfully"
        })),
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Delivery not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_match_summary(
    pool: web::Data<PgPool>,
    match_id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query(
        "SELECT 
            COUNT(*) as total_balls,
            SUM(runs_scored) as total_runs,
            COUNT(CASE WHEN is_wicket THEN 1 END) as total_wickets,
            MAX(over_number) as total_overs
         FROM progress
         WHERE match_id = $1"
    )
    .persistent(false)
    .bind(match_id.into_inner())
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let summary = serde_json::json!({
                "total_balls": row.get::<Option<i64>, _>(0).unwrap_or(0),
                "total_runs": row.get::<Option<i64>, _>(1).unwrap_or(0),
                "total_wickets": row.get::<Option<i64>, _>(2).unwrap_or(0),
                "total_overs": row.get::<Option<i32>, _>(3).unwrap_or(0)
            });
            HttpResponse::Ok().json(summary)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
