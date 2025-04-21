use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async fn connect_pg() -> PgPool {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL")
}
