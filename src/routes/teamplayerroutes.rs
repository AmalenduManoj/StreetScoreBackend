use actix_web::web;
use crate::handlers::team_players::{add_player_to_team, remove_player_from_team,get_team_created_by_user};
pub fn team_players_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/team_players")
            .route("", web::post().to(add_player_to_team))
            .route("", web::delete().to(remove_player_from_team))
            .route("/getteamme",web::get().to(get_team_created_by_user))
    );
}