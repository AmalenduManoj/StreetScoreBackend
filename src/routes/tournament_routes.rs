use actix_web::web;
use crate::handlers::team_tournament_handlers::{
    add_team_to_tournament,
    delete_team_from_tournament,
    get_team_in_tournament,
    get_tournaments_for_team,
};
use crate::handlers::tournament_handlers::{create_tournament, get_my_tournaments};

pub fn tournaments_routes_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tournaments")
            .route("", web::post().to(create_tournament))
            .route("/me/list", web::get().to(get_my_tournaments))
            .route("/{tournament_id}/teams", web::get().to(get_team_in_tournament))
            .route("/{tournament_id}/teams", web::post().to(add_team_to_tournament))
            .route("/{tournament_id}/teams", web::delete().to(delete_team_from_tournament))
            .route("/team/{team_id}", web::get().to(get_tournaments_for_team))
    );
}
