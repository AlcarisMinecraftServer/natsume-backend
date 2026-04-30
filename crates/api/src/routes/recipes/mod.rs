use std::sync::Arc;

use application::recipes::RecipeUsecase;
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
};
use domain::{recipes::Recipe, response::ApiResponse};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::audit::{actor_from_headers, insert_audit_log};

#[derive(Debug, serde::Deserialize)]
pub struct ListRecipeQuery {
    pub category: Option<String>,
}

pub async fn find_all_recipes(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<ListRecipeQuery>,
) -> impl IntoResponse {
    match usecase.find_all(query.category).await {
        Ok(recipes) => {
            #[derive(Debug, Clone, Serialize)]
            struct LastActor {
                id: Option<String>,
                username: Option<String>,
                global_name: Option<String>,
                avatar_url: Option<String>,
            }

            let ids: Vec<String> = recipes.iter().map(|r| r.id.clone()).collect();

            let mut map: std::collections::HashMap<String, LastActor> =
                std::collections::HashMap::new();
            if !ids.is_empty() {
                let rows = sqlx::query(
                    r#"
                    SELECT DISTINCT ON (recipe_id)
                        recipe_id,
                        actor_discord_id,
                        actor_username,
                        actor_global_name,
                        actor_avatar_url
                    FROM recipe_audit_logs
                    WHERE recipe_id = ANY($1::text[])
                    ORDER BY recipe_id, created_at DESC
                    "#,
                )
                .bind(&ids)
                .fetch_all(&pool)
                .await;

                match rows {
                    Ok(rows) => {
                        for row in rows {
                            let recipe_id: String = row.get("recipe_id");
                            let actor = LastActor {
                                id: row.get::<Option<String>, _>("actor_discord_id"),
                                username: row.get::<Option<String>, _>("actor_username"),
                                global_name: row.get::<Option<String>, _>("actor_global_name"),
                                avatar_url: row.get::<Option<String>, _>("actor_avatar_url"),
                            };
                            map.insert(recipe_id, actor);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("failed to load recipe_audit_logs: {e}");
                    }
                }
            }

            let mut out: Vec<Value> = Vec::with_capacity(recipes.len());
            for recipe in recipes {
                let mut v = serde_json::to_value(&recipe).unwrap_or(Value::Null);
                if let Value::Object(ref mut obj) = v {
                    let actor = map.remove(&recipe.id);
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
                "code": "fetch_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn find_recipes_by_id(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.find_by_id(&id).await {
        Ok(recipe) => Json(ApiResponse {
            status: 200,
            data: recipe,
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

pub async fn create_recipe(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(recipe): Json<Recipe>,
) -> impl IntoResponse {
    let recipe_id = recipe.id.clone();
    let after_data = serde_json::to_value(&recipe).ok();

    match usecase.create(recipe).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);

            if let Err(e) = sqlx::query(
                "INSERT INTO recipe_audit_logs (action, recipe_id, actor_discord_id, actor_username, actor_global_name, actor_avatar_url) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind("create")
            .bind(&recipe_id)
            .bind(actor.discord_id.as_deref())
            .bind(&actor.username)
            .bind(actor.global_name.as_deref())
            .bind(actor.avatar_url.as_deref())
            .execute(&pool)
            .await
            {
                tracing::warn!("failed to insert recipe_audit_logs: {e}");
            }

            insert_audit_log(
                &pool, "recipe", &recipe_id, "create", None, after_data, actor,
            )
            .await;

            (
                StatusCode::CREATED,
                Json(serde_json::json!({"status": 201})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "create_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn patch_recipe(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    let before_data = usecase
        .find_by_id(&id)
        .await
        .ok()
        .and_then(|r| serde_json::to_value(r).ok());

    match usecase.patch(&id, patch).await {
        Ok(_) => {
            let after_data = usecase
                .find_by_id(&id)
                .await
                .ok()
                .and_then(|r| serde_json::to_value(r).ok());

            let actor = actor_from_headers(&headers);

            if let Err(e) = sqlx::query(
                "INSERT INTO recipe_audit_logs (action, recipe_id, actor_discord_id, actor_username, actor_global_name, actor_avatar_url) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind("update")
            .bind(&id)
            .bind(actor.discord_id.as_deref())
            .bind(&actor.username)
            .bind(actor.global_name.as_deref())
            .bind(actor.avatar_url.as_deref())
            .execute(&pool)
            .await
            {
                tracing::warn!("failed to insert recipe_audit_logs: {e}");
            }

            insert_audit_log(
                &pool,
                "recipe",
                &id,
                "update",
                before_data,
                after_data,
                actor,
            )
            .await;

            Json(serde_json::json!({
                "status": 200,
                "message": "Recipe updated"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "update_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn delete_recipe(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let before_data = usecase
        .find_by_id(&id)
        .await
        .ok()
        .and_then(|r| serde_json::to_value(r).ok());

    match usecase.delete(&id).await {
        Ok(_) => {
            let actor = actor_from_headers(&headers);

            if let Err(e) = sqlx::query(
                "INSERT INTO recipe_audit_logs (action, recipe_id, actor_discord_id, actor_username, actor_global_name, actor_avatar_url) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind("delete")
            .bind(&id)
            .bind(actor.discord_id.as_deref())
            .bind(&actor.username)
            .bind(actor.global_name.as_deref())
            .bind(actor.avatar_url.as_deref())
            .execute(&pool)
            .await
            {
                tracing::warn!("failed to insert recipe_audit_logs: {e}");
            }

            insert_audit_log(&pool, "recipe", &id, "delete", before_data, None, actor).await;

            Json(serde_json::json!({
                "status": 200,
                "message": "Recipe deleted"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "delete_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
