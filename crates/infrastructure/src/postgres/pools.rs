use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

fn database_url_from_env() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER is not set");
    let pass = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD is not set");
    let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST is not set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT is not set");
    let db = env::var("POSTGRES_DATABASE").expect("POSTGRES_DATABASE is not set");

    format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, db)
}

pub async fn connect_pg() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url_from_env())
        .await
        .expect("Failed to connect to PostgreSQL")
}
