use actix_web::web;
use crate::handlers::progress_handlers::*;

pub fn progress_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/progress")
            // Create new delivery/ball entry
            .route("", web::post().to(create_progress))
            // Update delivery
            .route("/{id}", web::put().to(update_progress))
            // Delete delivery
            .route("/{id}", web::delete().to(delete_progress))
    );
}
