use actix_web::{web, App, HttpServer};

mod auth;
mod config;
mod handlers;
mod models;
mod routes;
use crate::routes::teamplayerroutes::team_players_routes_protected;
use crate::handlers::team_players::{get_players_in_team, get_teams_for_player};
use crate::config::db::create_pool;
use crate::handlers::auth_handlers::{signup, login, verify_auth, forgot_password, reset_password};
use crate::handlers::match_handlers::{get_matches, get_live_match, get_match_by_id};
use crate::auth::middleware::AuthMiddleware;
use crate::routes::match_routes::match_routes_protected;
use crate::routes::player_routes::player_routes_protected;
use crate::routes::team_routes::team_routes_protected;
use crate::routes::tournament_routes::tournaments_routes_protected;
use crate::handlers::tournament_handlers::{get_tournaments, get_tournament_by_id};
use crate::handlers::team_tournament_handlers::get_team_in_tournament;
use crate::handlers::team_handlers::{get_teams, get_team_by_id};
use crate::handlers::player_handler::{get_players, get_player_by_id};
use crate::handlers::progress_handlers::{get_progress_by_match, get_progress_by_over, get_progress_by_id, get_match_summary};
use crate::handlers::tournament_standing_handlers::{get_tournament_standings, get_team_standing, get_tournament_leaderboard, get_tournament_leaderboard_with_limit};
use crate::handlers::ranking_handlers::{get_batsman_rankings, get_batsman_ranking, get_bowler_rankings, get_bowler_ranking};
use crate::routes::progress_routes::progress_routes;
use crate::routes::tournament_standing_routes::tournament_standing_routes;
use crate::routes::ranking_routes::ranking_routes;
use crate::routes::tournament_match_routes::tournament_match_routes_protected;
use crate::handlers::match_lineup_handlers::{
    complete_match, get_match_lineup, get_match_player_stats, set_team_lineup, start_match,
};
use crate::handlers::tournament_match_handlers::{ get_tournament_matches, get_tournament_match, get_match_by_id as get_tournament_match_by_id};
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await;

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_origin("http://127.0.0.1:5173")
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
            )
            .app_data(web::Data::new(pool.clone()))
            .route("/auth/signup", web::post().to(signup))
            .route("/auth/login", web::post().to(login))
            .route("/auth/forgot-password", web::post().to(forgot_password))
            .route("/auth/reset-password", web::post().to(reset_password))
            .route("/matches", web::get().to(get_matches))
            .route("/matches/live", web::get().to(get_live_match))
            .route("/matches/{id}", web::get().to(get_match_by_id))
            .route("/tournaments", web::get().to(get_tournaments))
            .route("/tournaments/{id}", web::get().to(get_tournament_by_id))
            .route("/api/tournament/{id}", web::get().to(get_tournament_by_id))
            .route("/tournaments/{tournament_id}/teams", web::get().to(get_team_in_tournament))
            .route("/teams/get", web::get().to(get_teams))
            .route("/teams/{id}", web::get().to(get_team_by_id))
            .route("/players", web::get().to(get_players))
            .route("/players/stats/{id}", web::get().to(get_player_by_id))
            .route("/team_players/{team_id}", web::get().to(get_players_in_team))
            .route("/team_players/player/{player_id}", web::get().to(get_teams_for_player))
            .route("/api/progress/match/{match_id}", web::get().to(get_progress_by_match))
            .route("/api/progress/match/{match_id}/over/{over_number}", web::get().to(get_progress_by_over))
            .route("/api/progress/{id}", web::get().to(get_progress_by_id))
            .route("/api/progress/match/{match_id}/summary", web::get().to(get_match_summary))
            .route("/api/tournament/{tournament_id}/standings", web::get().to(get_tournament_standings))
            .route("/api/tournament/{tournament_id}/standings/{team_id}", web::get().to(get_team_standing))
            .route("/api/tournament/{tournament_id}/leaderboard", web::get().to(get_tournament_leaderboard))
            .route("/api/tournament/{tournament_id}/leaderboard/{limit}", web::get().to(get_tournament_leaderboard_with_limit))
            .route("/api/tournament/{tournament_id}/rankings/batsmen", web::get().to(get_batsman_rankings))
            .route("/api/tournament/{tournament_id}/rankings/batsmen/{player_id}", web::get().to(get_batsman_ranking))
            .route("/api/tournament/{tournament_id}/rankings/bowlers", web::get().to(get_bowler_rankings))
            .route("/api/tournament/{tournament_id}/rankings/bowlers/{player_id}", web::get().to(get_bowler_ranking))
            .route("/api/tournament/{tournament_id}/matches", web::get().to(get_tournament_matches))
            .route("/api/tournament/{tournament_id}/matches/{match_number}", web::get().to(get_tournament_match))
            .route("/api/tournament/match/{id}", web::get().to(get_tournament_match_by_id))
            .route("/api/matches/{match_id}/lineup", web::get().to(get_match_lineup))
            .route("/api/matches/{match_id}/player-stats", web::get().to(get_match_player_stats))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .wrap(
                        Cors::default()
                            .allowed_origin("http://localhost:5173")
                            .allowed_origin("http://127.0.0.1:5173")
                            .allowed_origin("http://127.0.0.1:3000")
                            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                            .allowed_headers(vec![
                                actix_web::http::header::AUTHORIZATION,
                                actix_web::http::header::CONTENT_TYPE,
                            ])
                            .supports_credentials()
                    )
                    .route("/auth/verify", web::get().to(verify_auth))
                    .route("/api/matches/{match_id}/lineup/{team_id}", web::put().to(set_team_lineup))
                    .route("/api/matches/{match_id}/start", web::post().to(start_match))
                    .route("/api/matches/{match_id}/complete", web::post().to(complete_match))
                    .configure(tournament_match_routes_protected)
                    .configure(match_routes_protected)  
                    .configure(player_routes_protected)
                    .configure(tournaments_routes_protected)
                    .configure(team_routes_protected)
                    .configure(team_players_routes_protected)
                    .configure(progress_routes)
                    .configure(tournament_standing_routes)
                    .configure(ranking_routes)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
