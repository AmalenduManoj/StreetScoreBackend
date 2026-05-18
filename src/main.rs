use actix_web::{web, App, HttpServer};

mod auth;
mod config;
mod handlers;
mod models;
mod routes;
use crate::routes::teamplayerroutes::team_players_routes_protected;
use crate::handlers::team_players::{get_players_in_team, get_teams_for_player};
use crate::config::db::create_pool;
use crate::handlers::auth_handlers::{signup, login, verify_auth};
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
use crate::routes::progress_routes::progress_routes;
use crate::routes::tournament_standing_routes::tournament_standing_routes;
use crate::routes::ranking_routes::ranking_routes;
use crate::routes::tournament_match_routes::tournament_match_routes_protected;
use crate::handlers::tournament_match_handlers::{ get_tournament_matches, get_tournament_match};
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
            // Public routes (no auth required)
            .route("/auth/signup", web::post().to(signup))
            .route("/auth/login", web::post().to(login))
            .route("/matches", web::get().to(get_matches))
            .route("/matches/live", web::get().to(get_live_match))
            .route("/matches/{id}", web::get().to(get_match_by_id))
            .route("/tournaments", web::get().to(get_tournaments))
            .route("/tournaments/{id}", web::get().to(get_tournament_by_id))
            .route("/tournaments/{tournament_id}/teams", web::get().to(get_team_in_tournament))
            .route("/teams", web::get().to(get_teams))
            .route("/teams/{id}", web::get().to(get_team_by_id))
            .route("/players", web::get().to(get_players))
            .route("/players/stats/{id}", web::get().to(get_player_by_id))
            .route("/team_players/{team_id}", web::get().to(get_players_in_team))
            .route("/team_players/player/{player_id}", web::get().to(get_teams_for_player))
            .route("/api/tournament/{tournament_id}/matches", web::get().to(get_tournament_matches))
            .route("/api/tournament/{tournament_id}/matches/{match_number}", web::get().to(get_tournament_match))
            .route("/api/tournament/match/{id}", web::get().to(get_match_by_id))
            .configure(progress_routes)
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
