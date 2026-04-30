use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub category: String,
    pub materials: Vec<RecipeMaterial>,
    pub product: RecipeProduct,
    pub is_hidden: bool,
    pub cooldown: Option<i32>,
    pub unlock_level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeMaterial {
    pub id: String,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeProduct {
    pub id: String,
    pub amount: i32,
}
