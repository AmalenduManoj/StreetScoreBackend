use actix_web::{web, App, HttpServer};

mod config;
mod handlers;
mod models;
mod routes;

use crate::config::db::create_pool;
use crate::routes::match_routes::match_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(match_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}