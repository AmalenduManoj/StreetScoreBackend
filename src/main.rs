use actix_web::{web, App, HttpServer};

mod auth;
mod config;
mod handlers;
mod models;
mod routes;

use crate::config::db::create_pool;
use crate::handlers::auth_handlers::{signup, login, verify_auth};
use crate::auth::middleware::AuthMiddleware;
use crate::routes::match_routes::match_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // Public routes (no auth required)
            .route("/auth/signup", web::post().to(signup))
            .route("/auth/login", web::post().to(login))
            // Protected routes (auth required)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .route("/auth/verify", web::get().to(verify_auth))
                    .configure(match_routes)  // Add this line back
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}