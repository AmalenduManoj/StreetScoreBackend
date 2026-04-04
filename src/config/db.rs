use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> PgPool {
    let connect_options = PgConnectOptions::from_str(database_url)
        .expect("Invalid DATABASE_URL")
        // Required for transaction-pooled connections (e.g., Supabase pooler on 6543).
        .statement_cache_capacity(0);

    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to the database")
}
