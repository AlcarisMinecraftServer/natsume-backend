use serde::{Serialize, Deserialize};
use super::{FoodData, ToolData, ArmorData};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub category: ItemCategory,
    pub version: i64,
    pub name: String,
    pub lore: Vec<String>,
    pub rarity: i16,
    pub max_stack: i16,
    pub custom_model_data: i32,
    pub price: Price,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub buy: i32,
    pub sell: i32,
    pub can_sell: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ItemData {
    Food(FoodData),
    Tool(ToolData),
    Armor(ArmorData),
}

impl std::fmt::Display for ItemCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ItemCategory::Food => "food",
                ItemCategory::Tool => "tool",
                ItemCategory::Armor => "armor",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ItemCategory {
    Food,
    Tool,
    Armor,
}
