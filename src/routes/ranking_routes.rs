use actix_web::web;
use crate::handlers::ranking_handlers::*;

pub fn ranking_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tournament")
            // ============== BATSMAN RANKINGS ==============
            // Get all batsman rankings for a tournament
            .route("/{tournament_id}/rankings/batsmen", web::get().to(get_batsman_rankings))
            // Get specific batsman ranking
            .route("/{tournament_id}/rankings/batsmen/{player_id}", web::get().to(get_batsman_ranking))
            // Create batsman ranking
            .route("/{tournament_id}/rankings/batsmen", web::post().to(create_batsman_ranking))
            // Update batsman ranking
            .route("/{tournament_id}/rankings/batsmen/{player_id}", web::put().to(update_batsman_ranking))
            
            // ============== BOWLER RANKINGS ==============
            // Get all bowler rankings for a tournament
            .route("/{tournament_id}/rankings/bowlers", web::get().to(get_bowler_rankings))
            // Get specific bowler ranking
            .route("/{tournament_id}/rankings/bowlers/{player_id}", web::get().to(get_bowler_ranking))
            // Create bowler ranking
            .route("/{tournament_id}/rankings/bowlers", web::post().to(create_bowler_ranking))
            // Update bowler ranking
            .route("/{tournament_id}/rankings/bowlers/{player_id}", web::put().to(update_bowler_ranking))
    );
}
