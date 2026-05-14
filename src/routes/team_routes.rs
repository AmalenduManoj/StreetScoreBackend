use actix_web::web;
use crate::handlers::team_handlers::{create_team};
pub fn team_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/teams")
            .route("", web::post().to(create_team))
    );
}