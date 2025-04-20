use axum::{Json, extract::Path, response::{Response, IntoResponse}};
use once_cell::sync::Lazy;

use crate::models::item::{Item, FoodItem, ToolItem};
use crate::utils::errors::not_found;

pub async fn list_food_items() -> Json<Vec<FoodItem>> {
    Json(FOOD_ITEMS.clone())
}

pub async fn list_tool_items() -> Json<Vec<ToolItem>> {
    Json(TOOL_ITEMS.clone())
}

pub async fn get_item_by_id(Path(id): Path<String>) -> Response {
    if let Some(f) = FOOD_ITEMS.iter().find(|i| i.id == id) {
        return Json(Item::Food(f.clone())).into_response();
    }
    if let Some(t) = TOOL_ITEMS.iter().find(|i| i.id == id) {
        return Json(Item::Tool(t.clone())).into_response();
    }
    not_found(&id)
}

static FOOD_ITEMS: Lazy<Vec<FoodItem>> = Lazy::new(|| {
    vec![FoodItem {
        id: "roasted_beef".into(),
        version: 1,
        display_name: "Roasted Beef".into(),
        rarity: 1,
        nutrition: 12,
        saturation: 6.4,
    }]
});

static TOOL_ITEMS: Lazy<Vec<ToolItem>> = Lazy::new(|| {
    vec![ToolItem {
        id: "iron_pickaxe".into(),
        version: 2,
        display_name: "Iron Pickaxe".into(),
        rarity: 2,
        max_damage: 250,
    }]
});
