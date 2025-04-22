use async_trait::async_trait;
use domain::files::FileMetadata;
use shared::error::AppResult;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait FileRepository {
    async fn insert_metadata(&self, metadata: &FileMetadata) -> AppResult<()>;
    async fn find_metadata(&self, id: &str) -> AppResult<FileMetadata>;
    async fn list_metadata(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>>;
    async fn delete_metadata(&self, id: &str) -> AppResult<()>;
}

pub struct PostgresFileRepository {
    pub pool: PgPool,
}

impl PostgresFileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn generate_cdn_url(user_id: &str, file_id: &str, filename: &str) -> String {
        format!(
            "https://cdn.alcaris.net/files/{}/{}/{}",
            user_id, file_id, filename
        )
    }
}

#[async_trait]
impl FileRepository for PostgresFileRepository {
    async fn insert_metadata(&self, metadata: &FileMetadata) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO files (id, user_id, filename, content_type, size, uploaded_at) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&metadata.id)
        .bind(&metadata.user_id)
        .bind(&metadata.filename)
        .bind(&metadata.content_type)
        .bind(metadata.size)
        .bind(metadata.uploaded_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_metadata(&self, id: &str) -> AppResult<FileMetadata> {
        let row = sqlx::query("SELECT * FROM files WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let user_id: String = row.get("user_id");
        let file_id: String = row.get("id");
        let filename: String = row.get("filename");

        Ok(FileMetadata {
            id: file_id.clone(),
            user_id: user_id.clone(),
            filename: filename.clone(),
            content_type: row.get("content_type"),
            size: row.get("size"),
            uploaded_at: row.get("uploaded_at"),
            url: Some(Self::generate_cdn_url(&user_id, &file_id, &filename)),
        })
    }

    async fn list_metadata(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>> {
        let rows = if let Some(ref user_id) = user_id {
            sqlx::query("SELECT * FROM files WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT * FROM files")
                .fetch_all(&self.pool)
                .await?
        };

        Ok(rows
            .into_iter()
            .map(|row| {
                let user_id: String = row.get("user_id");
                let file_id: String = row.get("id");
                let filename: String = row.get("filename");

                FileMetadata {
                    id: file_id.clone(),
                    user_id: user_id.clone(),
                    filename: filename.clone(),
                    content_type: row.get("content_type"),
                    size: row.get("size"),
                    uploaded_at: row.get("uploaded_at"),
                    url: Some(PostgresFileRepository::generate_cdn_url(
                        &user_id, &file_id, &filename,
                    )),
                }
            })
            .collect())
    }

    async fn delete_metadata(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
