mod routes;

use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    Json, Extension, Router,
    http::{StatusCode, Request},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use application::items::{ItemUsecase, ItemUsecaseImpl};
use infrastructure::postgres::{item_repository::PostgresItemRepository, pools::connect_pg};
use routes::items::{create_item, delete_item, find_all_items, find_item_by_id, patch_item};
use shared::error::not_found_handler;

pub async fn auth_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let secret = env::var("API_SECRET_KEY").unwrap_or_default();
    let auth = req.headers().get("Authorization").and_then(|v| v.to_str().ok());

    if let Some(auth_header) = auth {
        if auth_header == format!("Bearer {}", secret) {
            return Ok(next.run(req).await);
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "status": 401,
            "code": "unauthorized",
            "message": "Invalid or missing API token"
        })),
    )
        .into_response())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        )))
        .init();

    let port = env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Invalid port number in HTTP_PORT");

    let pool = connect_pg().await;
    let repo = PostgresItemRepository::new(pool);
    let usecase = Arc::new(ItemUsecaseImpl::new(repo)) as Arc<dyn ItemUsecase>;

    let app = Router::new()
        .route("/v1/items", get(find_all_items).post(create_item))
        .route(
            "/v1/items/{id}",
            get(find_item_by_id).patch(patch_item).delete(delete_item),
        )
        .layer(Extension(usecase))
        .layer(middleware::from_fn(auth_middleware))
        .fallback(not_found_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
