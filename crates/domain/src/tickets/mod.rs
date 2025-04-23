use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessage {
    pub sender: String,
    pub content: String,
    pub sent_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub status: String,
    pub messages: Vec<TicketMessage>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
