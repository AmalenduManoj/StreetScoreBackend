use actix_web::{HttpResponse, web};
use std::fs;

pub async fn get_api_docs() -> HttpResponse {
    match fs::read_to_string("API_ENDPOINTS.md") {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/markdown; charset=utf-8")
            .body(content),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to read docs: {}", e)),
    }
}
