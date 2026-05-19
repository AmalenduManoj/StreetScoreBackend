use actix_web::web;
use crate::handlers::ranking_handlers::*;

pub fn ranking_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tournament")
            // ============== BATSMAN RANKINGS ==============
            // Create batsman ranking
            .route("/{tournament_id}/rankings/batsmen", web::post().to(create_batsman_ranking))
            // Update batsman ranking
            .route("/{tournament_id}/rankings/batsmen/{player_id}", web::put().to(update_batsman_ranking))
            
            // ============== BOWLER RANKINGS ==============
            // Create bowler ranking
            .route("/{tournament_id}/rankings/bowlers", web::post().to(create_bowler_ranking))
            // Update bowler ranking
            .route("/{tournament_id}/rankings/bowlers/{player_id}", web::put().to(update_bowler_ranking))
    );
}
