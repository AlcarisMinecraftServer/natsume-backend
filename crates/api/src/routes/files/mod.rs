use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
};
use domain::response::ApiResponse;
use std::sync::Arc;

use application::files::FileUsecase;

#[derive(Debug, serde::Deserialize)]
pub struct FileListQuery {
    pub user_id: Option<String>,
}

pub async fn list_files(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Query(FileListQuery { user_id }): Query<FileListQuery>,
) -> impl IntoResponse {
    match usecase.find_all_files(user_id).await {
        Ok(files) => Json(ApiResponse {
            status: 200,
            data: files,
        })
        .into_response(),
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

pub async fn delete_file(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    match usecase.delete_file(&file_id).await {
        Ok(_) => Json(serde_json::json!({ "message": "File deleted" })).into_response(),
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

#[derive(Debug, serde::Deserialize)]
pub struct CreateUploadRequest {
    pub user_id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct UploadInitResponse {
    pub upload_id: String,
    pub file_id: String,
    pub user_id: String,
    pub key: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub part_size: i64,
    pub url: String,
}

fn cdn_url(user_id: &str, file_id: &str, filename: &str) -> String {
    format!(
        "https://cdn.alcaris.net/files/{}/{}/{}",
        user_id, file_id, filename
    )
}

pub async fn create_upload(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Json(req): Json<CreateUploadRequest>,
) -> impl IntoResponse {
    if req.user_id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": 400,
                "code": "bad_request",
                "message": "user_id is required"
            })),
        )
            .into_response();
    }

    match usecase
        .create_upload(&req.user_id, &req.filename, &req.content_type, req.size)
        .await
    {
        Ok(upload) => {
            let res = UploadInitResponse {
                upload_id: upload.upload_id,
                file_id: upload.file_id.clone(),
                user_id: upload.user_id.clone(),
                key: upload.key,
                filename: upload.filename.clone(),
                content_type: upload.content_type,
                size: upload.size,
                part_size: upload.part_size,
                url: cdn_url(&upload.user_id, &upload.file_id, &upload.filename),
            };
            Json(ApiResponse {
                status: 200,
                data: res,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "create_failed",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, serde::Serialize)]
pub struct UploadStatusResponse {
    pub upload_id: String,
    pub file_id: String,
    pub user_id: String,
    pub key: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub part_size: i64,
    pub url: String,
    pub parts: Vec<domain::files::FileUploadPart>,
}

pub async fn get_upload(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path(upload_id): Path<String>,
) -> impl IntoResponse {
    match usecase.get_upload(&upload_id).await {
        Ok((upload, parts)) => {
            let res = UploadStatusResponse {
                upload_id: upload.upload_id,
                file_id: upload.file_id.clone(),
                user_id: upload.user_id.clone(),
                key: upload.key,
                filename: upload.filename.clone(),
                content_type: upload.content_type,
                size: upload.size,
                part_size: upload.part_size,
                url: cdn_url(&upload.user_id, &upload.file_id, &upload.filename),
                parts,
            };
            Json(ApiResponse {
                status: 200,
                data: res,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": 404,
                "code": "not_found",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, serde::Serialize)]
pub struct PartUrlResponse {
    pub url: String,
}

pub async fn get_part_url(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path((upload_id, part_number)): Path<(String, i32)>,
) -> impl IntoResponse {
    match usecase.get_part_upload_url(&upload_id, part_number).await {
        Ok(url) => Json(ApiResponse {
            status: 200,
            data: PartUrlResponse { url },
        })
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": 404,
                "code": "not_found",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RegisterPartRequest {
    pub etag: String,
}

pub async fn register_part(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path((upload_id, part_number)): Path<(String, i32)>,
    Json(req): Json<RegisterPartRequest>,
) -> impl IntoResponse {
    match usecase
        .register_part(&upload_id, part_number, &req.etag)
        .await
    {
        Ok(_) => Json(serde_json::json!({ "message": "ok" })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "register_failed",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

pub async fn complete_upload(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    headers: HeaderMap,
    Path(upload_id): Path<String>,
) -> impl IntoResponse {
    let decode = |s: &str| {
        urlencoding::decode(s)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.to_string())
    };

    let get = |k: &str| {
        headers
            .get(k)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(decode)
    };

    let uploader_username = get("x-actor-discord-username");
    let uploader_global_name = get("x-actor-discord-global-name");
    let uploader_avatar_url = get("x-actor-discord-avatar");

    match usecase
        .complete_upload(
            &upload_id,
            uploader_username,
            uploader_global_name,
            uploader_avatar_url,
        )
        .await
    {
        Ok(meta) => Json(ApiResponse {
            status: 200,
            data: meta,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "complete_failed",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}

pub async fn abort_upload(
    Extension(usecase): Extension<Arc<dyn FileUsecase>>,
    Path(upload_id): Path<String>,
) -> impl IntoResponse {
    match usecase.abort_upload(&upload_id).await {
        Ok(_) => Json(serde_json::json!({ "message": "aborted" })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "abort_failed",
                "message": e.to_string(),
            })),
        )
            .into_response(),
    }
}
