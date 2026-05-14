use actix_web::web;
use crate::handlers::match_handlers::{create_match, update_match};

pub fn team_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/teams")
            .route("", web::post().to(create_team))
    );
}