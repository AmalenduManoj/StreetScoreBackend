use actix_web::web;
use crate::handlers::match_handlers::{create_match, get_matches};

pub fn match_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/matches")
            .route("", web::post().to(create_match))
            .route("", web::get().to(get_matches))
    );
}