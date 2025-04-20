use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct FoodItem {
    pub id: String,
    pub version: i64,
    pub display_name: String,
    pub rarity: i32,
    pub nutrition: i32,
    pub saturation: f32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ToolItem {
    pub id: String,
    pub version: i64,
    pub display_name: String,
    pub rarity: i32,
    pub max_damage: i32,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Item {
    Food(FoodItem),
    Tool(ToolItem),
}
