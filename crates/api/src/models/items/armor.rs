use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArmorData {
    pub slot: ArmorSlot,
    pub defense: i32,
    pub toughness: f32,
    pub knockback_resistance: f32,
    pub durability: i32,
    pub enchantable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}
