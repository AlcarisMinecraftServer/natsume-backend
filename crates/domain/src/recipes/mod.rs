use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub category: String,
    pub inputs: Vec<RecipeInput>,
    pub output: RecipeOutput,
    pub is_hidden: bool,
    pub cooldown: Option<i32>,
    pub unlock_level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeInput {
    pub item_id: String,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeOutput {
    pub item_id: String,
    pub amount: i32,
}
