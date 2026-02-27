use axum::{
    Json,
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
};
use domain::response::ApiResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub resource_type: String,
    pub resource_id: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub before_data: Option<Value>,
    pub after_data: Option<Value>,
    pub actor_discord_id: Option<String>,
    pub actor_username: String,
    pub actor_global_name: Option<String>,
    pub actor_avatar_url: Option<String>,
    pub created_at: String,
}

pub async fn list_audit_logs(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<AuditLogQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200);

    match sqlx::query(
        r#"
        SELECT
            id, resource_type, resource_id, action,
            before_data, after_data,
            actor_discord_id, actor_username, actor_global_name, actor_avatar_url,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM audit_logs
        WHERE resource_type = $1 AND resource_id = $2
        ORDER BY created_at DESC
        LIMIT $3
        "#,
    )
    .bind(&query.resource_type)
    .bind(&query.resource_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => {
            let entries: Vec<AuditLogEntry> = rows
                .into_iter()
                .map(|row| AuditLogEntry {
                    id: row.get("id"),
                    resource_type: row.get("resource_type"),
                    resource_id: row.get("resource_id"),
                    action: row.get("action"),
                    before_data: row.get("before_data"),
                    after_data: row.get("after_data"),
                    actor_discord_id: row.get("actor_discord_id"),
                    actor_username: row.get("actor_username"),
                    actor_global_name: row.get("actor_global_name"),
                    actor_avatar_url: row.get("actor_avatar_url"),
                    created_at: row.get("created_at"),
                })
                .collect();

            Json(ApiResponse {
                status: 200,
                data: entries,
            })
            .into_response()
        }
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
