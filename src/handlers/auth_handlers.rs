use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify};
use crate::auth::jwt::{generate_token, Claims};
use crate::models::users::User;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub async fn signup(
    pool: web::Data<PgPool>,
    body: web::Json<SignupRequest>,
) -> impl Responder {
 
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .persistent(false)
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await;

    match existing {
        Ok(Some(_)) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "User already exists"})),
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Database error"})),
        _ => {}
    }

    // Hash password
    let password_hash = match hash(&body.password, 4) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Hashing error"})),
    };

    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, created_at) VALUES ($1, $2, NOW()) RETURNING id, email, password_hash, created_at"
    )
    .persistent(false)
    .bind(&body.email)
    .bind(&password_hash)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(user) => {
            let token = match generate_token(user.id, user.email.clone()) {
                Ok(t) => t,
                Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Token generation failed"})),
            };
            HttpResponse::Ok().json(AuthResponse { token, user })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn login(
    pool: web::Data<PgPool>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let result = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1"
    )
    .persistent(false)
    .bind(&body.email)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => {
            match verify(&body.password, &user.password_hash) {
                Ok(true) => {
                    let token = match generate_token(user.id, user.email.clone()) {
                        Ok(t) => t,
                        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Token generation failed"})),
                    };
                    HttpResponse::Ok().json(AuthResponse { token, user })
                }
                _ => HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid credentials"})),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn verify_auth(req: HttpRequest) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing auth claims"})),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "user_id": claims.user_id,
        "email": claims.email,
        "authenticated": true
    }))
}