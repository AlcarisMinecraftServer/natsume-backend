use axum::http::HeaderMap;
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Actor {
    pub discord_id: Option<String>,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn actor_from_headers(headers: &HeaderMap) -> Actor {
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

pub async fn insert_audit_log(
    pool: &PgPool,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    before_data: Option<Value>,
    after_data: Option<Value>,
    actor: Actor,
) {
    if let Err(e) = sqlx::query(
        "INSERT INTO audit_logs \
         (resource_type, resource_id, action, before_data, after_data, \
          actor_discord_id, actor_username, actor_global_name, actor_avatar_url) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(resource_type)
    .bind(resource_id)
    .bind(action)
    .bind(before_data)
    .bind(after_data)
    .bind(actor.discord_id)
    .bind(actor.username)
    .bind(actor.global_name)
    .bind(actor.avatar_url)
    .execute(pool)
    .await
    {
        tracing::warn!("failed to insert audit_logs: {e}");
    }
}
