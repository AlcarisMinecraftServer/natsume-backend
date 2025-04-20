use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolData {
    pub tool_type: ToolType,
    pub max_damage: i32,
    pub rules: ToolRules,
    pub upgrades: Vec<Upgrade>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ToolType {
    Sword,
    Pickaxe,
    Axe,
    Shovel,
    Hoe,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolRules {
    pub default: Rule,
    pub conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub speed: f32,
    pub damage: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
    pub blocks: Vec<String>,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Upgrade {
    pub level: u8,
}
