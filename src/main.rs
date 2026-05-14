use actix_web::{web, App, HttpServer};

mod auth;
mod config;
mod handlers;
mod models;
mod routes;

use crate::config::db::create_pool;
use crate::handlers::auth_handlers::{signup, login, verify_auth};
use crate::handlers::match_handlers::{get_matches, get_live_match, get_match_by_id};
use crate::auth::middleware::AuthMiddleware;
use crate::routes::match_routes::match_routes_protected;
use crate::handlers::tournament_handlers::{get_tournaments, get_tournament_by_id};
use crate::handlers::team_handlers::{get_teams, get_team_by_id};
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
            .route("", web::get().to(get_tournaments))
            .route("/{id}", web::get().to(get_tournament_by_id))
            .route("", web::get().to(get_teams))
            .route("/{id}", web::get().to(get_team_by_id))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .route("/auth/verify", web::get().to(verify_auth))
                    .configure(match_routes_protected)  
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}