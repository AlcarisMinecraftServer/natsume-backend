mod models;
mod routes;
mod utils;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::fmt::init;

use routes::items::{list_food_items, list_tool_items, get_item_by_id};
use utils::errors::handle_404;

#[tokio::main]
async fn main() {
    init();

    let app = Router::new()
        .route("/v1/items/food", get(list_food_items))
        .route("/v1/items/tool", get(list_tool_items))
        .route("/v1/items/{id}", get(get_item_by_id))
        .fallback(handle_404);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

    println!("listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
