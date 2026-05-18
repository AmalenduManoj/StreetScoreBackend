use actix_web::web;
use crate::handlers::tournament_standing_handlers::*;

pub fn tournament_standing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tournament")
            // Get all standings for a tournament
            .route("/{tournament_id}/standings", web::get().to(get_tournament_standings))
            // Get specific team standing
            .route("/{tournament_id}/standings/{team_id}", web::get().to(get_team_standing))
            // Create standing
            .route("/{tournament_id}/standings", web::post().to(create_standing))
            // Update team standing
            .route("/{tournament_id}/standings/{team_id}", web::put().to(update_standing))
            // Get leaderboard (top teams)
            .route("/{tournament_id}/leaderboard", web::get().to(get_tournament_leaderboard))
            // Get leaderboard with limit
            .route("/{tournament_id}/leaderboard/{limit}", web::get().to(get_tournament_leaderboard))
    );
}
