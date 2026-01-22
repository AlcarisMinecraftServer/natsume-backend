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
    pub uploader_username: Option<String>,
    pub uploader_global_name: Option<String>,
    pub uploader_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadSession {
    pub upload_id: String,
    pub file_id: String,
    pub user_id: String,
    pub key: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub part_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadPart {
    pub part_number: i32,
    pub etag: String,
}
