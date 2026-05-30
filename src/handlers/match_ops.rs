use sqlx::{PgPool, Row};

use crate::models::r#match::Match;

pub async fn load_match(pool: &PgPool, match_id: i64) -> Result<Option<Match>, sqlx::Error> {
    sqlx::query_as::<_, Match>(
        "SELECT id, tournament_id, team1_id, team2_id, venue, total_overs,
                team1_score, team1_wickets, team1_overs::float4 AS team1_overs,
                team2_score, team2_wickets, team2_overs::float4 AS team2_overs, status
         FROM matches WHERE id = $1",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_optional(pool)
    .await
}

pub async fn sync_team1_from_progress(pool: &PgPool, match_id: i64) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        "SELECT
            COALESCE(SUM(runs_scored), 0) AS total_runs,
            COALESCE(COUNT(*) FILTER (WHERE is_wicket), 0) AS total_wickets,
            COALESCE(MAX(over_number), 0) AS total_overs
         FROM progress WHERE match_id = $1",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_one(pool)
    .await?;

    let runs: i64 = row.get("total_runs");
    let wickets: i64 = row.get("total_wickets");
    let overs: i32 = row.get("total_overs");

    sqlx::query(
        "UPDATE matches
         SET team1_score = $1, team1_wickets = $2, team1_overs = $3
         WHERE id = $4",
    )
    .persistent(false)
    .bind(runs as i32)
    .bind(wickets as i32)
    .bind(overs as f32)
    .bind(match_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn rebuild_match_player_stats(pool: &PgPool, match_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM match_player_stats WHERE match_id = $1")
        .persistent(false)
        .bind(match_id)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO match_player_stats (match_id, player_id, runs_scored, balls_faced, fours, sixes, is_out)
         SELECT match_id,
                batter_id,
                COALESCE(SUM(runs_scored), 0)::INT,
                COUNT(*)::INT,
                COALESCE(SUM(CASE WHEN runs_scored = 4 THEN 1 ELSE 0 END), 0)::INT,
                COALESCE(SUM(CASE WHEN runs_scored = 6 THEN 1 ELSE 0 END), 0)::INT,
                COALESCE(BOOL_OR(is_wicket), FALSE)
         FROM progress
         WHERE match_id = $1
         GROUP BY match_id, batter_id",
    )
    .persistent(false)
    .bind(match_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO match_player_stats (match_id, player_id, wickets_taken, balls_bowled, runs_conceded)
         SELECT match_id,
                bowler_id,
                COALESCE(COUNT(*) FILTER (WHERE is_wicket), 0)::INT,
                COUNT(*)::INT,
                COALESCE(SUM(runs_scored), 0)::INT
         FROM progress
         WHERE match_id = $1
         GROUP BY match_id, bowler_id
         ON CONFLICT (match_id, player_id) DO UPDATE
         SET wickets_taken = EXCLUDED.wickets_taken,
             balls_bowled = EXCLUDED.balls_bowled,
             runs_conceded = EXCLUDED.runs_conceded",
    )
    .persistent(false)
    .bind(match_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn bump_standing(
    pool: &PgPool,
    tournament_id: i64,
    team_id: i64,
    won: i32,
    lost: i32,
    points: i32,
    run_rate_delta: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE tournament_standing
         SET match_played = COALESCE(match_played, 0) + 1,
             wons = COALESCE(wons, 0) + $1,
             losses = COALESCE(losses, 0) + $2,
             points = COALESCE(points, 0) + $3,
             run_rate = COALESCE(run_rate, 0) + $4
         WHERE tournament_id = $5 AND team_id = $6",
    )
    .persistent(false)
    .bind(won)
    .bind(lost)
    .bind(points)
    .bind(run_rate_delta)
    .bind(tournament_id)
    .bind(team_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_batsman_ranking(
    pool: &PgPool,
    tournament_id: i64,
    player_id: i64,
    runs: i32,
    balls: i32,
    outs: i32,
) -> Result<(), sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE batsman_ranking
         SET runs = COALESCE(runs, 0) + $1,
             ball_faced = COALESCE(ball_faced, 0) + $2,
             no_of_outs = COALESCE(no_of_outs, 0) + $3
         WHERE tournament_id = $4 AND player_id = $5",
    )
    .persistent(false)
    .bind(runs)
    .bind(balls)
    .bind(outs)
    .bind(tournament_id)
    .bind(player_id)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO batsman_ranking (tournament_id, player_id, runs, ball_faced, no_of_outs)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .persistent(false)
        .bind(tournament_id)
        .bind(player_id)
        .bind(runs)
        .bind(balls)
        .bind(outs)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn upsert_bowler_ranking(
    pool: &PgPool,
    tournament_id: i64,
    player_id: i64,
    runs: i32,
    balls: i32,
    wickets: i32,
) -> Result<(), sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE bowler_ranking
         SET runs_given = COALESCE(runs_given, 0) + $1,
             ball_bowled = COALESCE(ball_bowled, 0) + $2,
             wickets = COALESCE(wickets, 0) + $3
         WHERE tournament_id = $4 AND player_id = $5",
    )
    .persistent(false)
    .bind(runs)
    .bind(balls)
    .bind(wickets)
    .bind(tournament_id)
    .bind(player_id)
    .execute(pool)
    .await?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO bowler_ranking (tournament_id, player_id, runs_given, ball_bowled, wickets)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .persistent(false)
        .bind(tournament_id)
        .bind(player_id)
        .bind(runs)
        .bind(balls)
        .bind(wickets)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn finalize_match(pool: &PgPool, match_id: i64) -> Result<Match, sqlx::Error> {
    sync_team1_from_progress(pool, match_id).await?;
    rebuild_match_player_stats(pool, match_id).await?;

    let m = load_match(pool, match_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    let team1_rr = if m.team1_overs > 0.0 {
        m.team1_score as f64 / f64::from(m.team1_overs)
    } else {
        0.0
    };
    let team2_rr = if m.team2_overs > 0.0 {
        m.team2_score as f64 / f64::from(m.team2_overs)
    } else {
        0.0
    };

    let (team1_won, team2_won, team1_lost, team2_lost, team1_pts, team2_pts) =
        if m.team1_score > m.team2_score {
            (1, 0, 0, 1, 2, 0)
        } else if m.team2_score > m.team1_score {
            (0, 1, 1, 0, 0, 2)
        } else {
            (0, 0, 0, 0, 1, 1)
        };

    bump_standing(pool, m.tournament_id, m.team1_id, team1_won, team1_lost, team1_pts, team1_rr).await?;
    bump_standing(pool, m.tournament_id, m.team2_id, team2_won, team2_lost, team2_pts, team2_rr).await?;

    let bat_rows = sqlx::query(
        "SELECT player_id, runs_scored, balls_faced, is_out
         FROM match_player_stats WHERE match_id = $1 AND balls_faced > 0",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_all(pool)
    .await?;

    for row in &bat_rows {
        upsert_batsman_ranking(
            pool,
            m.tournament_id,
            row.get("player_id"),
            row.get::<i32, _>("runs_scored"),
            row.get::<i32, _>("balls_faced"),
            if row.get::<bool, _>("is_out") { 1 } else { 0 },
        )
        .await?;
    }

    let bowl_rows = sqlx::query(
        "SELECT player_id, runs_conceded, balls_bowled, wickets_taken
         FROM match_player_stats WHERE match_id = $1 AND balls_bowled > 0",
    )
    .persistent(false)
    .bind(match_id)
    .fetch_all(pool)
    .await?;

    for row in &bowl_rows {
        upsert_bowler_ranking(
            pool,
            m.tournament_id,
            row.get("player_id"),
            row.get::<i32, _>("runs_conceded"),
            row.get::<i32, _>("balls_bowled"),
            row.get::<i32, _>("wickets_taken"),
        )
        .await?;
    }

    sqlx::query("UPDATE matches SET status = 'completed' WHERE id = $1")
        .persistent(false)
        .bind(match_id)
        .execute(pool)
        .await?;

    load_match(pool, match_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn lineup_count(pool: &PgPool, match_id: i64, team_id: i64) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM match_playing_xi WHERE match_id = $1 AND team_id = $2",
    )
    .persistent(false)
    .bind(match_id)
    .bind(team_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

pub async fn both_lineups_ready(pool: &PgPool, match_id: i64, team1_id: i64, team2_id: i64) -> Result<bool, sqlx::Error> {
    let c1 = lineup_count(pool, match_id, team1_id).await?;
    let c2 = lineup_count(pool, match_id, team2_id).await?;
    Ok(c1 == 11 && c2 == 11)
}

pub async fn player_in_lineup(
    pool: &PgPool,
    match_id: i64,
    player_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM match_playing_xi WHERE match_id = $1 AND player_id = $2)",
    )
    .persistent(false)
    .bind(match_id)
    .bind(player_id)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}
