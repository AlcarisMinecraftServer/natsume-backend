use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};
use tokio::sync::broadcast::Sender;

use crate::routes::ws::make_message;
use application::items::ItemUsecase;
use domain::{items::Item, response::ApiResponse};
use shared::error::item_not_found;

#[derive(Debug, Clone)]
struct Actor {
    discord_id: Option<String>,
    username: String,
    global_name: Option<String>,
    avatar_url: Option<String>,
}

fn actor_from_headers(headers: &HeaderMap) -> Actor {
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

    Actor {
        discord_id: get("x-actor-discord-id"),
        username: get("x-actor-discord-username").unwrap_or_else(|| "unknown".to_string()),
        global_name: get("x-actor-discord-global-name"),
        avatar_url: get("x-actor-discord-avatar"),
    }
}

async fn insert_audit_log(pool: &PgPool, action: &str, item_id: &str, actor: Actor) {
    if let Err(e) = sqlx::query(
        "INSERT INTO item_audit_logs (action, item_id, actor_discord_id, actor_username, actor_global_name, actor_avatar_url) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(action)
    .bind(item_id)
    .bind(actor.discord_id)
    .bind(actor.username)
    .bind(actor.global_name)
    .bind(actor.avatar_url)
    .execute(pool)
    .await
    {
        tracing::warn!("failed to insert item_audit_logs: {e}");
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ListItemQuery {
    pub category: Option<String>,
}

pub async fn find_all_items(
    Extension(usecase): Extension<Arc<dyn ItemUsecase>>,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<ListItemQuery>,
) -> impl IntoResponse {
    match usecase.find_all(query.category).await {
        Ok(items) => {
            #[derive(Debug, Clone, Serialize)]
            struct LastActor {
                id: Option<String>,
                username: Option<String>,
                global_name: Option<String>,
                avatar_url: Option<String>,
            }

            let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();

            let mut map: std::collections::HashMap<String, LastActor> =
                std::collections::HashMap::new();
            if !ids.is_empty() {
                let rows = sqlx::query(
                    r#"
                    SELECT DISTINCT ON (item_id)
                        item_id,
                        actor_discord_id,
                        actor_username,
                        actor_global_name,
                        actor_avatar_url
                    FROM item_audit_logs
                    WHERE item_id = ANY($1::text[])
                    ORDER BY item_id, created_at DESC
                    "#,
                )
                .bind(&ids)
                .fetch_all(&pool)
                .await;

                match rows {
                    Ok(rows) => {
                        for row in rows {
                            let item_id: String = row.get("item_id");
                            let actor = LastActor {
                                id: row.get::<Option<String>, _>("actor_discord_id"),
                                username: row.get::<Option<String>, _>("actor_username"),
                                global_name: row.get::<Option<String>, _>("actor_global_name"),
                                avatar_url: row.get::<Option<String>, _>("actor_avatar_url"),
                            };
                            map.insert(item_id, actor);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("failed to load item_audit_logs: {e}");
                    }
                }
            }

            let mut out: Vec<Value> = Vec::with_capacity(items.len());
            for item in items {
                let mut v = serde_json::to_value(&item).unwrap_or(Value::Null);
                if let Value::Object(ref mut obj) = v {
                    let actor = map.remove(&item.id);
                    obj.insert(
                        "last_actor".to_string(),
                        actor
                            .map(|a| serde_json::to_value(a).unwrap_or(Value::Null))
                            .unwrap_or(Value::Null),
                    );
                }
                out.push(v);
            }

            Json(ApiResponse {
                status: 200,
                data: out,
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
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(item): Json<Item>,
) -> impl IntoResponse {
    let item_id = item.id.clone();

    match usecase.create(item).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);
            let actor_name = actor.username.clone();
            insert_audit_log(&pool, "create", &item_id, actor).await;

            let msg = make_message("create", "item", &actor_name, "web");
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
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    match usecase.patch(&id, patch).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);
            let actor_name = actor.username.clone();
            insert_audit_log(&pool, "update", &id, actor).await;

            let msg = make_message("update", "item", &actor_name, "web");
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
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.delete(&id).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);
            let actor_name = actor.username.clone();
            insert_audit_log(&pool, "delete", &id, actor).await;

            let msg = make_message("delete", "item", &actor_name, "web");
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
