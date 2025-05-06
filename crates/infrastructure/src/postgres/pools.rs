use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL is not set")
}

pub async fn connect_pg() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url())
        .await
        .expect("Failed to connect to PostgreSQL");

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
