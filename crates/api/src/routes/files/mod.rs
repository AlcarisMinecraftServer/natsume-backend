use application::files::FileUsecase;
use axum::{
    Json,
    extract::{Extension, Multipart, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use domain::response::ApiResponse;
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
pub struct FileListQuery {
    pub user_id: Option<String>,
}

pub async fn upload_file(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Query(FileListQuery { user_id }): Query<FileListQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().unwrap_or("file").to_string();
        let content_type = field
            .content_type()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let data = field.bytes().await.unwrap();

        let meta = usecase
            .upload_file(
                &user_id.clone().unwrap_or_default(),
                &filename,
                &content_type,
                data,
            )
            .await;

        return match meta {
            Ok(m) => Json(ApiResponse {
                status: 200,
                data: m,
            })
            .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 500,
                    "code": "create_failed",
                    "message": e.to_string()
                })),
            )
                .into_response(),
        };
    }

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "message": "No file found" })),
    )
        .into_response()
}

pub async fn get_file_by_id(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    match usecase.get_file_by_id(&file_id).await {
        Ok(meta) => Json(ApiResponse {
            status: 200,
            data: meta,
        })
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": 404,
                "code": "not_found",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn find_all_files(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Query(FileListQuery { user_id }): Query<FileListQuery>,
) -> impl IntoResponse {
    match usecase.find_all_files(user_id).await {
        Ok(files) => Json(files).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "fetch_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn delete_file(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    match usecase.delete_file(&file_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "File deleted" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "delete_failed",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}
