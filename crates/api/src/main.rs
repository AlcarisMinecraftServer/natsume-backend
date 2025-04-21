mod models;
mod routes;
mod utils;

use axum::{Extension, Router, routing::get};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

use routes::items::{create_item, delete_item, get_item_by_id, list_items, update_item_partial};
use utils::{db::connect_pg, errors::not_found_handler};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            )),
        )
        .init();

    dotenv().ok();

    let port = env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Invalid port number in HTTP_PORT");

    let pool = connect_pg().await;

    let app = Router::new()
        .route("/v1/items", get(list_items).post(create_item))
        .route(
            "/v1/items/{id}",
            get(get_item_by_id)
                .patch(update_item_partial)
                .delete(delete_item),
        )
        .layer(Extension(pool))
        .fallback(not_found_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
