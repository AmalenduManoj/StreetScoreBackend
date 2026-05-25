use crate::auth::jwt::{Claims, generate_token};
use crate::models::users::User;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

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

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

fn generate_reset_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn hash_reset_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn required_env(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{} must be set", name))
}

async fn send_password_reset_email(to_email: &str, reset_url: &str) -> Result<(), String> {
    let smtp_host = required_env("SMTP_HOST")?;
    let smtp_username = required_env("SMTP_USERNAME")?;
    let smtp_password = required_env("SMTP_PASSWORD")?;
    let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| smtp_username.clone());
    let smtp_tls = std::env::var("SMTP_TLS")
        .unwrap_or_else(|_| "starttls".to_string())
        .to_lowercase();
    let default_port = if smtp_tls == "tls" { 465 } else { 587 };
    let smtp_port = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(default_port);

    let email = Message::builder()
        .from(
            smtp_from
                .parse::<Mailbox>()
                .map_err(|_| "SMTP_FROM must be a valid email address".to_string())?,
        )
        .to(to_email
            .parse::<Mailbox>()
            .map_err(|_| "Recipient email is invalid".to_string())?)
        .subject("Reset your CricScore password")
        .body(format!(
            "Hi,\n\nUse this link to reset your CricScore password:\n\n{}\n\nThis link expires in 60 minutes. If you did not request this, you can ignore this email.\n\nCricScore",
            reset_url
        ))
        .map_err(|e| e.to_string())?;

    let credentials = Credentials::new(smtp_username, smtp_password);
    let mailer = match smtp_tls.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
            .map_err(|e| e.to_string())?
            .port(smtp_port)
            .credentials(credentials)
            .build(),
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_host)
            .port(smtp_port)
            .credentials(credentials)
            .build(),
        _ => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .map_err(|e| e.to_string())?
            .port(smtp_port)
            .credentials(credentials)
            .build(),
    };

    mailer.send(email).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn signup(pool: web::Data<PgPool>, body: web::Json<SignupRequest>) -> impl Responder {
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .persistent(false)
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await;

    match existing {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "User already exists"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database error"}));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match hash(&body.password, 4) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Hashing error"}));
        }
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
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"error": "Token generation failed"}));
                }
            };
            HttpResponse::Ok().json(AuthResponse { token, user })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn login(pool: web::Data<PgPool>, body: web::Json<LoginRequest>) -> impl Responder {
    let result = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1",
    )
    .persistent(false)
    .bind(&body.email)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => match verify(&body.password, &user.password_hash) {
            Ok(true) => {
                let token = match generate_token(user.id, user.email.clone()) {
                    Ok(t) => t,
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Token generation failed"}));
                    }
                };
                HttpResponse::Ok().json(AuthResponse { token, user })
            }
            _ => HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid credentials"})),
        },
        Ok(None) => {
            HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
    }
}

pub async fn forgot_password(
    pool: web::Data<PgPool>,
    body: web::Json<ForgotPasswordRequest>,
) -> impl Responder {
    let email = body.email.trim();

    if email.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Email is required"}));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE LOWER(email) = LOWER($1)",
    )
    .persistent(false)
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await;

    let Ok(user) = user else {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Database error"}));
    };

    let Some(user) = user else {
        return HttpResponse::Ok().json(serde_json::json!({
            "message": "If an account exists for this email, a password reset link has been generated."
        }));
    };

    let token = generate_reset_token();
    let token_hash = hash_reset_token(&token);
    let expires_at = Utc::now() + Duration::hours(1);

    let result = sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .persistent(false)
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool.get_ref())
    .await;

    if result.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to create reset token"}));
    }

    let frontend_reset_url = match std::env::var("FRONTEND_RESET_PASSWORD_URL") {
        Ok(url) => url,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "FRONTEND_RESET_PASSWORD_URL must be set"}));
        }
    };
    let reset_url = format!("{}?token={}", frontend_reset_url.trim_end_matches('/'), token);

    if let Err(e) = send_password_reset_email(&user.email, &reset_url).await {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": format!("Failed to send reset email: {}", e)}));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "If an account exists for this email, a password reset link has been sent."
    }))
}

pub async fn reset_password(
    pool: web::Data<PgPool>,
    body: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    let token = body.token.trim();

    if token.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Reset token is required"}));
    }

    if body.password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Password must be at least 8 characters"}));
    }

    let token_hash = hash_reset_token(token);

    let reset_record = sqlx::query_as::<_, (i64, i64)>(
        "SELECT id, user_id
         FROM password_reset_tokens
         WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()",
    )
    .persistent(false)
    .bind(&token_hash)
    .fetch_optional(pool.get_ref())
    .await;

    let reset_record = match reset_record {
        Ok(Some(record)) => record,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid or expired reset token"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database error"}));
        }
    };

    let password_hash = match hash(&body.password, 4) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Hashing error"}));
        }
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database error"}));
        }
    };

    let password_update = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .persistent(false)
        .bind(&password_hash)
        .bind(reset_record.1)
        .execute(&mut *tx)
        .await;

    if password_update.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to reset password"}));
    }

    let token_update =
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .persistent(false)
            .bind(reset_record.0)
            .execute(&mut *tx)
            .await;

    if token_update.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to reset password"}));
    }

    if tx.commit().await.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Failed to reset password"}));
    }

    HttpResponse::Ok().json(serde_json::json!({"message": "Password reset successfully"}))
}

pub async fn verify_auth(req: HttpRequest) -> impl Responder {
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing auth claims"}));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "user_id": claims.user_id,
        "email": claims.email,
        "authenticated": true
    }))
}
