use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::{PgPool, Row};

use crate::handlers::match_ops::{
    both_lineups_ready, finalize_match, load_match, player_in_lineup, rebuild_match_player_stats,
    sync_team1_from_progress,
};
use crate::handlers::tournament_auth::{
    claims_from_request, forbidden, is_tournament_creator_for_match, unauthorized,
};

#[derive(Deserialize)]
pub struct SetLineupRequest {
    pub player_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct CompleteMatchRequest {
    pub team2_score: Option<i32>,
    pub team2_wickets: Option<i32>,
    pub team2_overs: Option<f32>,
}

pub async fn get_match_lineup(
    pool: web::Data<PgPool>,
    match_id: web::Path<i64>,
) -> HttpResponse {
    let match_id = match_id.into_inner();

    let rows = sqlx::query(
        "SELECT xi.team_id, xi.player_id, p.name, p.role
         FROM match_playing_xi xi
         JOIN players p ON p.id = xi.player_id
         WHERE xi.match_id = $1
         ORDER BY xi.team_id, p.name",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let lineup: Vec<_> = rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "team_id": row.get::<i64, _>("team_id"),
                        "player_id": row.get::<i64, _>("player_id"),
                        "name": row.get::<String, _>("name"),
                        "role": row.get::<Option<String>, _>("role"),
                    })
                })
                .collect();
            HttpResponse::Ok().json(lineup)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn set_team_lineup(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<(i64, i64)>,
    data: web::Json<SetLineupRequest>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Some(claims) => claims,
        None => return unauthorized(),
    };

    let (match_id, team_id) = path.into_inner();

    if data.player_ids.len() != 11 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Playing XI must contain exactly 11 players"
        }));
    }

    let allowed = match is_tournament_creator_for_match(pool.get_ref(), match_id, claims.user_id).await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if !allowed {
        return forbidden("Only the tournament creator can set the playing XI");
    }

    let m = match load_match(pool.get_ref(), match_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({ "error": "Match not found" }));
        }
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if m.status != "scheduled" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Playing XI can only be changed while the match is scheduled"
        }));
    }

    if team_id != m.team1_id && team_id != m.team2_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Team is not part of this match"
        }));
    }

    let valid_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM team_player_registry
         WHERE team_id = $1 AND player_id = ANY($2)",
    )
    .persistent(false)
    .bind(team_id)
    .bind(&data.player_ids)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);

    if valid_count != 11 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "All 11 players must belong to the team squad"
        }));
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if let Err(e) = sqlx::query(
        "DELETE FROM match_playing_xi WHERE match_id = $1 AND team_id = $2",
    )
    .persistent(false)
    .bind(match_id)
    .bind(team_id)
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }));
    }

    for player_id in &data.player_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO match_playing_xi (match_id, team_id, player_id) VALUES ($1, $2, $3)",
        )
        .persistent(false)
        .bind(match_id)
        .bind(team_id)
        .bind(player_id)
        .execute(&mut *tx)
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }));
        }
    }

    match tx.commit().await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Playing XI saved",
            "team_id": team_id,
            "player_count": 11
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn start_match(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    match_id: web::Path<i64>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Some(claims) => claims,
        None => return unauthorized(),
    };

    let match_id = match_id.into_inner();

    let allowed = match is_tournament_creator_for_match(pool.get_ref(), match_id, claims.user_id).await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if !allowed {
        return forbidden("Only the tournament creator can start the match");
    }

    let m = match load_match(pool.get_ref(), match_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({ "error": "Match not found" }));
        }
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if m.status != "scheduled" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Only scheduled matches can be started"
        }));
    }

    let ready = match both_lineups_ready(pool.get_ref(), match_id, m.team1_id, m.team2_id).await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if !ready {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Both teams must have a playing XI of 11 players before starting"
        }));
    }

    match sqlx::query("UPDATE matches SET status = 'live' WHERE id = $1")
        .persistent(false)
        .bind(match_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "message": "Match started", "status": "live" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn complete_match(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    match_id: web::Path<i64>,
    body: Option<web::Json<CompleteMatchRequest>>,
) -> HttpResponse {
    let claims = match claims_from_request(&req) {
        Some(claims) => claims,
        None => return unauthorized(),
    };

    let match_id = match_id.into_inner();

    let allowed = match is_tournament_creator_for_match(pool.get_ref(), match_id, claims.user_id).await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    };

    if !allowed {
        return forbidden("Only the tournament creator can complete the match");
    }

    if let Some(body) = body {
        if let (Some(score), Some(wickets), Some(overs)) =
            (body.team2_score, body.team2_wickets, body.team2_overs)
        {
            if let Err(e) = sqlx::query(
                "UPDATE matches SET team2_score = $1, team2_wickets = $2, team2_overs = $3 WHERE id = $4",
            )
            .persistent(false)
            .bind(score)
            .bind(wickets)
            .bind(overs)
            .bind(match_id)
            .execute(pool.get_ref())
            .await
            {
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() }));
            }
        }
    }

    match finalize_match(pool.get_ref(), match_id).await {
        Ok(m) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Match completed. Standings and player stats updated.",
            "match": m
        })),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Match not found" }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn get_match_player_stats(
    pool: web::Data<PgPool>,
    match_id: web::Path<i64>,
) -> HttpResponse {
    let match_id = match_id.into_inner();

    let rows = sqlx::query(
        "SELECT mps.player_id, p.name, p.role,
                mps.runs_scored, mps.balls_faced, mps.fours, mps.sixes, mps.is_out,
                mps.wickets_taken, mps.balls_bowled, mps.runs_conceded
         FROM match_player_stats mps
         JOIN players p ON p.id = mps.player_id
         WHERE mps.match_id = $1
         ORDER BY mps.runs_scored DESC, mps.wickets_taken DESC",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let stats: Vec<_> = rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "player_id": row.get::<i64, _>("player_id"),
                        "name": row.get::<String, _>("name"),
                        "role": row.get::<Option<String>, _>("role"),
                        "runs_scored": row.get::<i32, _>("runs_scored"),
                        "balls_faced": row.get::<i32, _>("balls_faced"),
                        "fours": row.get::<i32, _>("fours"),
                        "sixes": row.get::<i32, _>("sixes"),
                        "is_out": row.get::<bool, _>("is_out"),
                        "wickets_taken": row.get::<i32, _>("wickets_taken"),
                        "balls_bowled": row.get::<i32, _>("balls_bowled"),
                        "runs_conceded": row.get::<i32, _>("runs_conceded"),
                    })
                })
                .collect();
            HttpResponse::Ok().json(stats)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

pub async fn validate_progress_players(
    pool: &PgPool,
    match_id: i64,
    batter_id: i64,
    bowler_id: i64,
) -> Result<(), HttpResponse> {
    let batter_ok = player_in_lineup(pool, match_id, batter_id).await.unwrap_or(false);
    let bowler_ok = player_in_lineup(pool, match_id, bowler_id).await.unwrap_or(false);

    if !batter_ok || !bowler_ok {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Batter and bowler must be in the match playing XI"
        })));
    }

    Ok(())
}

pub async fn after_progress_recorded(pool: &PgPool, match_id: i64) -> Result<(), sqlx::Error> {
    sync_team1_from_progress(pool, match_id).await?;
    rebuild_match_player_stats(pool, match_id).await?;
    Ok(())
}
