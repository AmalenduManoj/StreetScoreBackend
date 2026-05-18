use actix_web::{HttpResponse, web};
use sqlx::{PgPool, Row};
use crate::models::progress::progress;
use chrono::Utc;

// Create a new ball/delivery entry
pub async fn create_progress(
    pool: web::Data<PgPool>,
    data: web::Json<progress>,
) -> HttpResponse {
    let result = sqlx::query(
        "INSERT INTO progress (match_id, batter_id, bowler_id, runs_scored, is_wicket, over_number, ball_number, commentary, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id"
    )
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

// Get all deliveries in a match
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

// Get deliveries in a specific over
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

// Get specific delivery
pub async fn get_progress_by_id(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query_as::<_, progress>(
        "SELECT id, match_id, batter_id, bowler_id, runs_scored, is_wicket, over_number, ball_number, commentary, created_at
         FROM progress
         WHERE id = $1"
    )
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

// Update delivery (for corrections)
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

// Delete delivery
pub async fn delete_progress(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> HttpResponse {
    let result = sqlx::query("DELETE FROM progress WHERE id = $1")
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

// Get summary stats for a match
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
