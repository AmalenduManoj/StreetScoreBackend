use actix_web::web;
use crate::handlers::tournament_standing_handlers::*;

pub fn tournament_standing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tournament")
            // Create standing
            .route("/{tournament_id}/standings", web::post().to(create_standing))
            // Update team standing
            .route("/{tournament_id}/standings/{team_id}", web::put().to(update_standing))
    );
}
