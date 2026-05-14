use actix_web::web;
use crate::handlers::match_handlers::{create_match, get_live_match, get_matches,get_match_by_id,update_match};

pub fn match_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/matches")
            .route("", web::post().to(create_match))
            .route("", web::get().to(get_matches))
            .route("/live", web::get().to(get_live_match))
            .route("/{id}", web::get().to(get_match_by_id))
            .route("/update/{id}", web::put().to(update_match))
    );
}