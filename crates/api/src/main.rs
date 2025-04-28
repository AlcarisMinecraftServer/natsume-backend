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
    files::{FileUsecase, FileUsecaseImpl}, items::{ItemUsecase, ItemUsecaseImpl}, recipes::{RecipeUsecase, RecipeUsecaseImpl}, status::{StatusUsecase, StatusUsecaseImpl}, tickets::{TicketUsecase, TicketUsecaseImpl}
};
use infrastructure::{
    postgres::pools::connect_pg,
    repositorys::{
        file::PostgresFileRepository, item::PostgresItemRepository, recipe::PostgresRecipeRepository, status::PostgresStatusRepository, ticket::PostgresTicketRepository
    }, status_watcher::start_status_watcher,
};
use routes::status::{get_status, list_status};
use routes::items::{create_item, delete_item, find_all_items, find_item_by_id, patch_item};
use routes::recipes::{
    create_recipe, delete_recipe, find_all_recipes, find_recipes_by_id, patch_recipe,
};
use routes::tickets::{create_ticket, find_ticket_by_id, list_tickets};
use routes::{
    files::{delete_file, find_all_files, get_file_by_id, upload_file},
    tickets::{delete_ticket, patch_ticket},
};
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

    let file_repo = PostgresFileRepository::new(pool.clone());
    let file_usecase = Arc::new(FileUsecaseImpl::new(file_repo)) as Arc<dyn FileUsecase>;

    let item_repo = PostgresItemRepository::new(pool.clone());
    let item_usecase = Arc::new(ItemUsecaseImpl::new(item_repo)) as Arc<dyn ItemUsecase>;

    let recipe_repo = PostgresRecipeRepository::new(pool.clone());
    let recipe_usecase = Arc::new(RecipeUsecaseImpl::new(recipe_repo)) as Arc<dyn RecipeUsecase>;

    let status_repo = PostgresStatusRepository::new(pool.clone());
    let status_usecase = Arc::new(StatusUsecaseImpl::new(status_repo)) as Arc<dyn StatusUsecase>;

    let ticket_repo: PostgresTicketRepository = PostgresTicketRepository::new(pool.clone());
    let ticket_usecase = Arc::new(TicketUsecaseImpl::new(ticket_repo)) as Arc<dyn TicketUsecase>;

    start_status_watcher(pool.clone()).await.unwrap();

    let app = Router::new()
        .route("/v1/items", get(find_all_items).post(create_item))
        .route(
            "/v1/items/{id}",
            get(find_item_by_id).patch(patch_item).delete(delete_item),
        )
        .layer(Extension(item_usecase))
        .route("/v1/recipes", get(find_all_recipes).post(create_recipe))
        .route(
            "/v1/recipes/{id}",
            get(find_recipes_by_id)
                .patch(patch_recipe)
                .delete(delete_recipe),
        )
        .layer(Extension(recipe_usecase))
        .route("/v1/files", get(find_all_files).post(upload_file))
        .route("/v1/files/{id}", get(get_file_by_id).delete(delete_file))
        .layer(Extension(file_usecase))
        .route("/v1/status", get(list_status))
        .route("/v1/status/{server_id}", get(get_status))
        .layer(Extension(status_usecase))
        .route("/v1/tickets", get(list_tickets).post(create_ticket))
        .route(
            "/v1/tickets/{id}",
            get(find_ticket_by_id)
                .patch(patch_ticket)
                .delete(delete_ticket),
        )
        .layer(Extension(ticket_usecase))
        .layer(middleware::from_fn(auth_middleware))
        .fallback(not_found_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
