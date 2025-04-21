use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

fn database_url_from_env() -> String {
    let user = env::var("POSTGRES_USER").unwrap();
    let pass = env::var("POSTGRES_PASSWORD").unwrap();
    let host = env::var("POSTGRES_HOST").unwrap();
    let port = env::var("POSTGRES_PORT").unwrap();
    let db   = env::var("POSTGRES_DATABASE").unwrap();

    format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, db)
}

pub async fn connect_pg() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url_from_env())
        .await
        .expect("Failed to connect to PostgreSQL")
}
