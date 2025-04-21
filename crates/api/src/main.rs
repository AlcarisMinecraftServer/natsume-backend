mod models;
mod routes;
mod utils;

use axum::{routing::get, Extension, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::fmt::init;

use routes::items::{list_items, create_item, get_item_by_id, update_item_partial, delete_item};
use utils::{db::connect_pg, errors::not_found};

#[tokio::main]
async fn main() {
    init();

    dotenv().ok();

    let pool = connect_pg().await;

    let app = Router::new()
        .route(
            "/v1/items",
            get(list_items)
            .post(create_item)
        )
        .route(
            "/v1/items/{id}",
            get(get_item_by_id)
            .patch(update_item_partial)
            .delete(delete_item)
        )
        .layer(Extension(pool))
        .fallback(not_found);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

    println!("listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
