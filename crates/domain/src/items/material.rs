use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialData {
    pub magic_materials: Vec<String>,
}
