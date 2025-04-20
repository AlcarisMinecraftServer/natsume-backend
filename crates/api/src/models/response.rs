use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: u16,
    pub data: T,
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
}
