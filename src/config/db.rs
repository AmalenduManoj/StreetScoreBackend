use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
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

pub async fn create_pool(database_url: &str) -> PgPool {
    let url = if database_url.contains(":6543") && !database_url.contains("pgbouncer=true") {
        if database_url.contains('?') {
            format!("{database_url}&pgbouncer=true")
        } else {
            format!("{database_url}?pgbouncer=true")
        }
    } else {
        database_url.to_string()
    };

    let mut connect_options = PgConnectOptions::from_str(&url).expect("Invalid DATABASE_URL");

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
        .expect("Failed to connect to the database")
}
