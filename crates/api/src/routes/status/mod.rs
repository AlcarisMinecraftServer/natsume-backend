use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    Json,
};
use application::status::StatusUsecase;
use domain::response::ApiResponse;
use std::sync::Arc;
use axum::http::StatusCode;

pub async fn get_status(
    Extension(usecase): Extension<Arc<dyn StatusUsecase>>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    match usecase.find_by_id(&server_id).await {
        Ok(status) => Json(ApiResponse {
            status: 200,
            data: status,
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

pub async fn list_status(
    Extension(usecase): Extension<Arc<dyn StatusUsecase>>,
) -> impl IntoResponse {
    match usecase.find_all().await {
        Ok(status) => Json(ApiResponse {
            status: 200,
            data: status,
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
