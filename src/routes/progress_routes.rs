use actix_web::web;
use crate::handlers::progress_handlers::*;

pub fn progress_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/progress")
            // Create new delivery/ball entry
            .route("", web::post().to(create_progress))
            // Get all deliveries in a match
            .route("/match/{match_id}", web::get().to(get_progress_by_match))
            // Get deliveries in a specific over
            .route("/match/{match_id}/over/{over_number}", web::get().to(get_progress_by_over))
            // Get specific delivery
            .route("/{id}", web::get().to(get_progress_by_id))
            // Update delivery
            .route("/{id}", web::put().to(update_progress))
            // Delete delivery
            .route("/{id}", web::delete().to(delete_progress))
            // Get match summary stats
            .route("/match/{match_id}/summary", web::get().to(get_match_summary))
    );
}
