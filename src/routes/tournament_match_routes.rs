use actix_web::web;
use crate::handlers::tournament_match_handlers::*;

pub fn tournament_match_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tournament")
            // Create tournament match
            .route("/{tournament_id}/matches", web::post().to(create_tournament_match))
            // Get all matches in tournament
            // Update tournament match
            .route("/match/{id}", web::put().to(update_tournament_match))
            // Delete tournament match
            .route("/match/{id}", web::delete().to(delete_tournament_match))
            // Get match with full details
            .route("/{tournament_id}/matches/{match_number}/details", web::get().to(get_match_full_details))
    );
}
