use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use tokio::sync::broadcast::Sender;

use crate::routes::ws::make_message;
use application::items::ItemUsecase;
use domain::{items::Item, response::ApiResponse};
use shared::error::item_not_found;

#[derive(Debug, serde::Deserialize)]
pub struct ListItemQuery {
    pub category: Option<String>,
}

pub async fn find_all_items(
    Extension(usecase): Extension<Arc<dyn ItemUsecase>>,
    Query(query): Query<ListItemQuery>,
) -> impl IntoResponse {
    match usecase.find_all(query.category).await {
        Ok(items) => Json(ApiResponse {
            status: 200,
            data: items,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_fetch_error",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn find_item_by_id(
    Extension(usecase): Extension<Arc<dyn ItemUsecase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.find_by_id(&id).await {
        Ok(item) => Json(ApiResponse {
            status: 200,
            data: item,
        })
        .into_response(),
        Err(_) => item_not_found(&id),
    }
}

pub async fn create_item(
    Extension(usecase): Extension<Arc<dyn ItemUsecase>>,
    Extension(tx): Extension<Arc<Sender<String>>>,
    Json(item): Json<Item>,
) -> impl IntoResponse {
    match usecase.create(item).await {
        Ok(_) => {
            // WebSocket通知
            let msg = make_message("create", "item", "admin", "web");
            let _ = tx.send(msg);

            (
                StatusCode::CREATED,
                Json(ApiResponse {
                    status: 201,
                    data: "Item created",
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_insert_error",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

pub async fn patch_item(
    Extension(usecase): Extension<Arc<dyn ItemUsecase>>,
    Extension(tx): Extension<Arc<Sender<String>>>,
    Path(id): Path<String>,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    match usecase.patch(&id, patch).await {
        Ok(_) => {
            let msg = make_message("update", "item", "admin", "web");
            let _ = tx.send(msg);

            Json(serde_json::json!({
                "status": 200,
                "message": "Item updated"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_update_error",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

pub async fn delete_item(
    Extension(usecase): Extension<Arc<dyn ItemUsecase>>,
    Extension(tx): Extension<Arc<Sender<String>>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.delete(&id).await {
        Ok(_) => {
            let msg = make_message("delete", "item", "admin", "web");
            let _ = tx.send(msg);

            Json(serde_json::json!({
                "status": 200,
                "message": "Item deleted"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_delete_error",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}
