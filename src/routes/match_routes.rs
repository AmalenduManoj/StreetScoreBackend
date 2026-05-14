use actix_web::web;
use crate::handlers::match_handlers::{create_match, update_match};

pub fn match_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/matches")
            .route("", web::post().to(create_match))
            .route("/update/{id}", web::put().to(update_match))
    );
}