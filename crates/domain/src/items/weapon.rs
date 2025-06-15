use serde::{Deserialize, Serialize};

use crate::items::{Buff, Effect};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponData {
    pub weapon_type: WeaponType,
    pub required_level: u32,
    pub max_modification: u32,
    pub durability: u32,
    pub base: Base,
    pub upgrades: Vec<WeaponUpgrade>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WeaponType {
    Sword,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attributes {
    pub attack_damage: f32,
    pub movement_speed: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub experience_bonus: f32,
    pub drop_rate_bonus: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Base {
    pub attributes: Attributes,
    pub effects: Vec<Effect>,
    pub buffs: Vec<Buff>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponUpgrade {
    // TODO: 後で
}
