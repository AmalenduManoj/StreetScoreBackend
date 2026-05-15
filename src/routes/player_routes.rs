use actix_web::web;
use crate::handlers::player_handler::{create_player, update_player,get_player_me};

pub fn player_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/players")
            .route("", web::post().to(create_player))
            .route("/update/{id}", web::put().to(update_player))
            .route("/me", web::get().to(get_player_me))
           
    );
}