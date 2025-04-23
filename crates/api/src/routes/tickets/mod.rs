use application::tickets::TicketUsecase;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use domain::tickets::Ticket;
use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
pub struct TicketQuery {
    pub user_id: Option<String>,
}

pub async fn list_tickets(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Query(TicketQuery { user_id }): Query<TicketQuery>,
) -> impl IntoResponse {
    match usecase.find_all(user_id).await {
        Ok(tickets) => Json(tickets).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn find_ticket_by_id(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.find_by_id(&id).await {
        Ok(ticket) => Json(ticket).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn create_ticket(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Json(ticket): Json<Ticket>,
) -> impl IntoResponse {
    match usecase.create(ticket).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "message": "Created" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn patch_ticket(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Path(id): Path<String>,
    Json(ticket): Json<Ticket>,
) -> impl IntoResponse {
    match usecase.update(&id, ticket).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Updated" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_ticket(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.delete(&id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Deleted" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}
