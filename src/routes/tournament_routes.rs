use actix_web::web;
use crate::handlers::tournament_handlers::{create_tournament};
pub fn tournaments_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tournaments")
            .route("", web::post().to(create_tournament))
    );
}