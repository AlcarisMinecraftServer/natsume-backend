use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

pub fn not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "code": "not_found",
            "status": 404,
            "message": format!("item '{}' not found", id)
        })),
    ).into_response()
}

pub async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "code": "not_found",
            "status": 404,
            "message": "The requested resource was not found"
        })),
    )
}
