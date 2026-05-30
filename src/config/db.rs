use actix_web::rt::time::sleep;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};
use std::str::FromStr;
use std::time::Duration;

async fn reset_connection(conn: &mut sqlx::PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("DISCARD ALL")
        .persistent(false)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, String> {
    let url = if database_url.contains(":6543") && !database_url.contains("pgbouncer=true") {
        if database_url.contains('?') {
            format!("{database_url}&pgbouncer=true")
        } else {
            format!("{database_url}?pgbouncer=true")
        }
    } else {
        database_url.to_string()
    };

    let mut connect_options = PgConnectOptions::from_str(&url)
        .map_err(|err| format!("Invalid DATABASE_URL: {err}"))?;

    if (url.contains("supabase.co") || url.contains("supabase.com"))
        && !url.contains("sslmode=")
    {
        connect_options = connect_options.ssl_mode(PgSslMode::Require);
    }

    connect_options = connect_options
        .statement_cache_capacity(0)
        .options([("statement_cache_mode", "off")]);

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .test_before_acquire(true)
        .before_acquire(|conn, _meta| {
            Box::pin(async move {
                reset_connection(conn).await?;
                Ok(true)
            })
        })
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                reset_connection(conn).await?;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await
        .map_err(|err| format!("Failed to connect to the database: {err}"))
}

pub async fn connect_pool_with_retry(database_url: &str, attempts: u32) -> Result<PgPool, String> {
    let mut last_error = String::from("no attempts made");

    for attempt in 1..=attempts {
        match create_pool(database_url).await {
            Ok(pool) => {
                if attempt > 1 {
                    eprintln!("Database connected on attempt {attempt}");
                }
                return Ok(pool);
            }
            Err(err) => {
                last_error = err;
                eprintln!("Database connection attempt {attempt}/{attempts} failed: {last_error}");
                if attempt < attempts {
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    Err(last_error)
}
