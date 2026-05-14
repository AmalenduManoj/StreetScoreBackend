use actix_web::web;
use crate::handlers::match_handlers::{create_match, update_match};

pub fn tournaments_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tournaments")
            .route("", web::post().to(create_tournament))
    );
}