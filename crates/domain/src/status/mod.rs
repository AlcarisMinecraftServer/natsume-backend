use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Players {
    pub online: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusRecord {
    pub online: bool,
    pub latency: Option<i32>,
    pub players: Option<Players>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub id: String,
    pub online: bool,
    pub latency: Option<i32>,
    pub players: Option<Players>,
    pub timestamp: i64,
    pub history: Vec<StatusRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSummary {
    pub id: String,
    pub online: bool,
    pub latency: Option<i32>,
    pub players: Option<Players>,
    pub timestamp: i64,
    pub history: Vec<StatusRecord>,
}
