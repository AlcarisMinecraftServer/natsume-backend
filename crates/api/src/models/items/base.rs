use serde::{Serialize, Deserialize};
use super::{FoodData, ToolData, ArmorData};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub category: ItemCategory,
    pub version: i64,
    pub name: String,
    pub lore: Vec<String>,
    pub rarity: i8,
    pub max_stack: i32,
    pub custom_model_data: i32,
    pub price: Price,
    pub data: ItemData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub buy: i32,
    pub sell: i32,
    pub can_sell: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "category", content = "data")]
pub enum ItemData {
    Food(FoodData),
    Tool(ToolData),
    Armor(ArmorData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ItemCategory {
    Food,
    Tool,
    Armor,
}
