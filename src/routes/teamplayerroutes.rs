use actix_web::web;
use crate::handlers::team_players::{add_player_to_team, get_players_in_team, remove_player_from_team, get_teams_for_player};
pub fn team_players_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/team_players")
            .route("", web::post().to(add_player_to_team))
            .route("", web::delete().to(remove_player_from_team))
    );
}