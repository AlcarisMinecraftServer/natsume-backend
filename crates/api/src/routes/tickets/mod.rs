use application::tickets::TicketUsecase;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use domain::tickets::Ticket;
use sqlx::PgPool;
use std::sync::Arc;

use crate::audit::{actor_from_headers, insert_audit_log};

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
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(ticket): Json<Ticket>,
) -> impl IntoResponse {
    let ticket_id = ticket.id.clone();
    let after_data = serde_json::to_value(&ticket).ok();

    match usecase.create(ticket).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);
            insert_audit_log(
                &pool, "ticket", &ticket_id, "create", None, after_data, actor,
            )
            .await;

            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "message": "Created" })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn patch_ticket(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(ticket): Json<Ticket>,
) -> impl IntoResponse {
    let before_data = usecase
        .find_by_id(&id)
        .await
        .ok()
        .and_then(|t| serde_json::to_value(t).ok());
    let after_data = serde_json::to_value(&ticket).ok();

    match usecase.update(&id, ticket).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);
            insert_audit_log(
                &pool,
                "ticket",
                &id,
                "update",
                before_data,
                after_data,
                actor,
            )
            .await;

            (
                StatusCode::OK,
                Json(serde_json::json!({ "message": "Updated" })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_ticket(
    Extension(usecase): Extension<Arc<dyn TicketUsecase>>,
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let before_data = usecase
        .find_by_id(&id)
        .await
        .ok()
        .and_then(|t| serde_json::to_value(t).ok());

    match usecase.delete(&id).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);
            insert_audit_log(&pool, "ticket", &id, "delete", before_data, None, actor).await;

            (
                StatusCode::OK,
                Json(serde_json::json!({ "message": "Deleted" })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "message": e.to_string() })),
        )
            .into_response(),
    }
}
