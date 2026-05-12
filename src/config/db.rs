use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> PgPool {
    let mut connect_options = PgConnectOptions::from_str(database_url)
        .expect("Invalid DATABASE_URL");
    
    connect_options = connect_options.statement_cache_capacity(0);
    
    PgPoolOptions::new()
        .max_connections(5)
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to the database")
}
