use axum::{
    extract::{Path, Query}, http::StatusCode, response::{IntoResponse, Response}, Extension, Json
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::models::{response::ApiResponse, items::{
    Item, ItemCategory
}};
use crate::utils::errors::item_not_found;

pub async fn list_items(
    Query(query): Query<ListItemQuery>,
    Extension(pool): Extension<PgPool>,
) -> Json<ApiResponse<Vec<Item>>> {
    let category_filter = query.category.clone();

    let rows = sqlx::query("SELECT * FROM items")
        .fetch_all(&pool)
        .await
        .expect("DB error");

    let mut items = vec![];

    for row in rows {
        let category_str: String = row.get("category");

        if let Some(ref filter) = category_filter {
            if &category_str.to_lowercase() != &filter.to_lowercase() {
                continue;
            }
        }

        let item = Item {
            id: row.get("id"),
            version: row.get("version"),
            name: row.get("name"),
            category: match category_str.to_lowercase().as_str() {
                "food" => ItemCategory::Food,
                "tool" => ItemCategory::Tool,
                "armor" => ItemCategory::Armor,
                _ => continue,
            },
            lore: serde_json::from_value(row.get::<Value, _>("lore")).unwrap_or_default(),
            rarity: row.get("rarity"),
            max_stack: row.get("max_stack"),
            custom_model_data: row.get("custom_model_data"),
            price: serde_json::from_value(row.get("price")).unwrap(),
            data: row.get("data"),
        };

        items.push(item);
    }

    Json(ApiResponse {
        status: 200,
        data: items,
    })
}

#[derive(Debug, Deserialize)]
pub struct ListItemQuery {
    pub category: Option<String>,
}

pub async fn get_item_by_id(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Response {
    let row = sqlx::query("SELECT * FROM items WHERE id = $1")
        .bind(&id)
        .fetch_optional(&pool)
        .await;

    match row {
        Ok(Some(row)) => {
            let category_str: String = row.get("category");
            let category = match category_str.to_lowercase().as_str() {
                "food" => ItemCategory::Food,
                "tool" => ItemCategory::Tool,
                "armor" => ItemCategory::Armor,
                _ => return item_not_found(&id),
            };

            let item = Item {
                id: row.get("id"),
                version: row.get("version"),
                name: row.get("name"),
                category,
                lore: serde_json::from_value(row.get::<Value, _>("lore")).unwrap_or_default(),
                rarity: row.get("rarity"),
                max_stack: row.get("max_stack"),
                custom_model_data: row.get("custom_model_data"),
                price: serde_json::from_value(row.get("price")).unwrap(),
                data: row.get("data"),
            };

            Json(item).into_response()
        }

        Ok(None) => item_not_found(&id),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_error",
                "message": format!("Failed to query item: {}", e),
            })),
        )
            .into_response(),
    }
}

pub async fn create_item(
    Extension(pool): Extension<PgPool>,
    Json(item): Json<Item>,
) -> impl IntoResponse {
    let result = sqlx::query(
        r#"
        INSERT INTO items (
            id, version, name, category,
            lore, rarity, max_stack, custom_model_data,
            price, data
        ) VALUES (
            $1, $2, $3, $4,
            to_jsonb($5), $6, $7, $8,
            to_jsonb($9), to_jsonb($10)
        )
        "#,
    )
    .bind(&item.id)
    .bind(item.version)
    .bind(&item.name)
    .bind(item.category.to_string())
    .bind(serde_json::to_value(&item.lore).unwrap())
    .bind(item.rarity)
    .bind(item.max_stack)
    .bind(item.custom_model_data)
    .bind(serde_json::to_value(&item.price).unwrap())
    .bind(item.data.clone()) 
    .execute(&pool)
    .await;

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ApiResponse {
                status: 201,
                data: (),
            }),
        )
            .into_response(),

        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_insert_error",
                "message": format!("Failed to insert item: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn update_item_partial(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>,
    Json(patch): Json<serde_json::Value>
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE items SET data = data || $1 WHERE id = $2")
        .bind(patch)
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
            "message": "Item updated"
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_update_error",
                "message": format!("Failed to update item: {}", e)
            }))
        ).into_response(),
    }
}

pub async fn delete_item(
    Path(id): Path<String>,
    Extension(pool): Extension<PgPool>
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
            "message": "Item deleted"
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "db_delete_error",
                "message": format!("Failed to delete item: {}", e)
            }))
        ).into_response(),
    }
}