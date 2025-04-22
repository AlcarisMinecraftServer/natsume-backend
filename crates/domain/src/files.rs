use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub uploaded_at: chrono::NaiveDateTime,
    pub url: Option<String>,
}
