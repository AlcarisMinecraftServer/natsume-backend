use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::models::{response::ApiResponse, items::{
    FoodData, Item, ItemCategory, ItemData, Price, Rule, Rules, ToolData, ToolType
}};
use crate::utils::errors::item_not_found;

pub async fn list_items(Query(query): Query<ListItemQuery>) -> Json<ApiResponse<Vec<Item>>> {
    let items = ITEMS
        .iter()
        .filter(|i| match query.category {
            Some(ref cat) => match cat.as_str() {
                "food" => matches!(i.category, ItemCategory::Food),
                "tool" => matches!(i.category, ItemCategory::Tool),
                "armor" => matches!(i.category, ItemCategory::Armor),
                _ => false,
            },
            None => true,
        })
        .cloned()
        .collect();

    Json(ApiResponse {
        status: 200,
        data: items,
    })
}

#[derive(Debug, Deserialize)]
pub struct ListItemQuery {
    pub category: Option<String>,
}

pub async fn get_item_by_id(Path(id): Path<String>) -> Response {
    if let Some(item) = ITEMS.iter().find(|i| i.id == id) {
        return Json(item.clone()).into_response();
    }
    item_not_found(&id)
}

pub async fn create_item(Json(_item): Json<Item>) -> impl IntoResponse {
    // TODO: 永続化があればここで保存する
    (StatusCode::CREATED, Json(serde_json::json!({ "status": "created" })))
}

pub async fn update_item_partial(Path(id): Path<String>, Json(_patch): Json<serde_json::Value>) -> impl IntoResponse {
    // TODO: IDに一致するアイテムを見つけて部分更新する（現状は静的なため実処理は未対応）
    Json(serde_json::json!({
        "status": "ok",
        "message": format!("PATCH not persisted yet for item '{}'", id)
    }))
}

pub async fn delete_item(Path(id): Path<String>) -> impl IntoResponse {
    // TODO: 実際の削除処理はDB実装後に対応
    Json(serde_json::json!({
        "status": "ok",
        "message": format!("DELETE not persisted yet for item '{}'", id)
    }))
}

// 仮の静的データ
static ITEMS: Lazy<Vec<Item>> = Lazy::new(|| {
    vec![
        Item {
            id: "roasted_beef".to_string(),
            version: 1,
            name: "Roasted Beef".to_string(),
            category: ItemCategory::Food,
            lore: vec!["A delicious cooked beef.".to_string()],
            rarity: 1,
            max_stack: 64,
            custom_model_data: 1001,
            price: Price {
                buy: 50,
                sell: 25,
                can_sell: true,
            },
            data: ItemData::Food(FoodData {
                nutrition: 12,
                saturation: 6.4,
                can_always_eat: false,
                eat_seconds: 1.6,
                effects: vec![],
                attributes: vec![],
                buffs: vec![],
            }),
        },
        Item {
            id: "iron_pickaxe".to_string(),
            version: 2,
            name: "Iron Pickaxe".to_string(),
            category: ItemCategory::Tool,
            lore: vec!["Used for mining stone blocks.".to_string()],
            rarity: 2,
            max_stack: 1,
            custom_model_data: 2001,
            price: Price {
                buy: 100,
                sell: 40,
                can_sell: true,
            },
            data: ItemData::Tool(ToolData {
                tool_type: ToolType::Pickaxe,
                max_damage: 250,
                rules: Rules {
                    default: Rule {
                        speed: 1.0,
                        damage: 2,
                    },
                    conditions: vec![],
                },
                upgrades: vec![],
            }),
        },
    ]
});
