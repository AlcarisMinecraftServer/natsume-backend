mod routes;

use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json, Router,
    body::Body,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use application::{
    items::{ItemUsecase, ItemUsecaseImpl},
    recipes::{RecipeUsecase, RecipeUsecaseImpl},
};
use infrastructure::{
    postgres::pools::connect_pg,
    repositorys::{item::PostgresItemRepository, recipe::PostgresRecipeRepository},
};
use routes::items::{create_item, delete_item, find_all_items, find_item_by_id, patch_item};
use routes::recipes::{create_recipe, find_all_recipes, find_recipes_by_id};
use shared::error::not_found_handler;

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, Response> {
    let secret = env::var("API_SECRET_KEY").unwrap_or_default();
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

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

    let repo_item = PostgresItemRepository::new(pool.clone());
    let usecase_item = Arc::new(ItemUsecaseImpl::new(repo_item)) as Arc<dyn ItemUsecase>;

    let repo_recipe = PostgresRecipeRepository::new(pool.clone());
    let usecase_recipe = Arc::new(RecipeUsecaseImpl::new(repo_recipe)) as Arc<dyn RecipeUsecase>;

    let app = Router::new()
        .route("/v1/items", get(find_all_items).post(create_item))
        .route(
            "/v1/items/{id}",
            get(find_item_by_id).patch(patch_item).delete(delete_item),
        )
        .route("/v1/recipes", get(find_all_recipes).post(create_recipe))
        .route("/v1/recipes/{id}", get(find_recipes_by_id))
        .layer(Extension(usecase_item))
        .layer(Extension(usecase_recipe))
        .layer(middleware::from_fn(auth_middleware))
        .fallback(not_found_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
