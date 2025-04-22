use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::response::ApiErrorResponse;

pub type AppResult<T> = Result<T, anyhow::Error>;

pub fn item_not_found(id: &str) -> Response {
    let body = ApiErrorResponse {
        status: 404,
        code: "item_not_found",
        message: format!("Item '{}' not found", id),
    };

    (StatusCode::NOT_FOUND, Json(body)).into_response()
}

pub async fn not_found_handler() -> impl IntoResponse {
    let body = ApiErrorResponse {
        status: 404,
        code: "not_found",
        message: "The requested resource was not found".to_string(),
    };

    (StatusCode::NOT_FOUND, Json(body)).into_response()
}
