use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FoodData {
    pub nutrition: i32,
    pub saturation: f32,
    pub can_always_eat: bool,
    pub eat_seconds: f32,
    pub effects: Vec<Effect>,
    pub attributes: Vec<Attribute>,
    pub buffs: Vec<Buff>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Effect {
    pub effect: String,
    pub duration: i32,
    pub amplifier: u8,
    pub chance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute {
    pub attribute: String,
    pub operation: String,
    pub value: f32,
    pub duration: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Buff {
    pub kind: String,
    pub duration: i32,
    pub amount: f32,
}
