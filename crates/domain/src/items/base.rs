use super::{ArmorData, FoodData, ToolData, WeaponData};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: String,
    pub category: ItemCategory,
    pub version: i64,
    pub name: String,
    pub lore: Vec<String>,
    pub rarity: i16,
    pub max_stack: i16,
    #[serde(
        default,
        deserialize_with = "deserialize_custom_model_data_single_compatible"
    )]
    pub custom_model_data: Option<CustomModelData>,
    #[serde(default)]
    pub item_model: Option<String>,
    #[serde(default)]
    pub tooltip_style: Option<String>,
    pub price: Price,
    pub tags: Vec<Tag>,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum CustomModelData {
    Floats(Vec<f32>),
    Flags(Vec<bool>),
    Strings(Vec<String>),
    Colors(Vec<i32>),
}

fn deserialize_custom_model_data_single_compatible<'de, D>(
    deserializer: D,
) -> Result<Option<CustomModelData>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CompatibleData {
        Single(CustomModelData),
        LegacyInt(()),
    }

    let data: Option<CompatibleData> = Option::deserialize(deserializer)?;

    match data {
        Some(CompatibleData::Single(c)) => Ok(Some(c)),
        Some(CompatibleData::LegacyInt(_)) => Ok(None),
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub buy: i32,
    pub sell: i32,
    pub can_sell: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub label: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ItemData {
    Weapon(WeaponData),
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
                ItemCategory::Weapon => "weapon",
                ItemCategory::Material => "material",
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
    Weapon,
    Material,
}
