use actix_web::web;

use crate::handlers::match_lineup_handlers::{
    complete_match, set_team_lineup, start_match,
};

pub fn match_management_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/matches")
            .route("/{match_id}/lineup/{team_id}", web::put().to(set_team_lineup))
            .route("/{match_id}/start", web::post().to(start_match))
            .route("/{match_id}/complete", web::post().to(complete_match)),
    );
}
