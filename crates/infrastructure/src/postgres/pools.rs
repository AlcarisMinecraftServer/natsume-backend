use sqlx::{Connection, PgConnection, PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL is not set")
}

pub async fn connect_pg() -> anyhow::Result<PgPool> {
    let mut conn = PgConnection::connect(&database_url()).await?;
    MIGRATOR.run(&mut conn).await?;
    drop(conn);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url())
        .await?;

    Ok(pool)
}
