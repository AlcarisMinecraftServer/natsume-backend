use async_trait::async_trait;
use domain::files::{FileMetadata, FileUploadPart, FileUploadSession};
use shared::error::AppResult;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait FileRepository {
    async fn insert_metadata(&self, metadata: &FileMetadata) -> AppResult<()>;
    async fn find_metadata(&self, id: &str) -> AppResult<FileMetadata>;
    async fn list_metadata(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>>;
    async fn delete_metadata(&self, id: &str) -> AppResult<()>;

    async fn create_upload(&self, upload: &FileUploadSession) -> AppResult<()>;
    async fn find_upload(&self, upload_id: &str) -> AppResult<FileUploadSession>;
    async fn list_upload_parts(&self, upload_id: &str) -> AppResult<Vec<FileUploadPart>>;
    async fn upsert_upload_part(
        &self,
        upload_id: &str,
        part_number: i32,
        etag: &str,
    ) -> AppResult<()>;
    async fn delete_upload(&self, upload_id: &str) -> AppResult<()>;
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
            "INSERT INTO files (id, user_id, filename, content_type, size, uploaded_at, uploader_username, uploader_global_name, uploader_avatar_url) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&metadata.id)
        .bind(&metadata.user_id)
        .bind(&metadata.filename)
        .bind(&metadata.content_type)
        .bind(metadata.size)
        .bind(metadata.uploaded_at)
        .bind(&metadata.uploader_username)
        .bind(&metadata.uploader_global_name)
        .bind(&metadata.uploader_avatar_url)
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
            uploader_username: row.get::<Option<String>, _>("uploader_username"),
            uploader_global_name: row.get::<Option<String>, _>("uploader_global_name"),
            uploader_avatar_url: row.get::<Option<String>, _>("uploader_avatar_url"),
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
                    uploader_username: row.get::<Option<String>, _>("uploader_username"),
                    uploader_global_name: row.get::<Option<String>, _>("uploader_global_name"),
                    uploader_avatar_url: row.get::<Option<String>, _>("uploader_avatar_url"),
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

    async fn create_upload(&self, upload: &FileUploadSession) -> AppResult<()> {
        sqlx::query(
             "INSERT INTO file_uploads (upload_id, file_id, user_id, key, filename, content_type, size, part_size, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'in_progress')",
         )
         .bind(&upload.upload_id)
         .bind(&upload.file_id)
         .bind(&upload.user_id)
         .bind(&upload.key)
         .bind(&upload.filename)
         .bind(&upload.content_type)
         .bind(upload.size)
         .bind(upload.part_size)
         .execute(&self.pool)
         .await?;
        Ok(())
    }

    async fn find_upload(&self, upload_id: &str) -> AppResult<FileUploadSession> {
        let row = sqlx::query("SELECT * FROM file_uploads WHERE upload_id = $1")
            .bind(upload_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(FileUploadSession {
            upload_id: row.get("upload_id"),
            file_id: row.get("file_id"),
            user_id: row.get("user_id"),
            key: row.get("key"),
            filename: row.get("filename"),
            content_type: row.get("content_type"),
            size: row.get("size"),
            part_size: row.get("part_size"),
        })
    }

    async fn list_upload_parts(&self, upload_id: &str) -> AppResult<Vec<FileUploadPart>> {
        let rows = sqlx::query(
             "SELECT part_number, etag FROM file_upload_parts WHERE upload_id = $1 ORDER BY part_number",
         )
         .bind(upload_id)
         .fetch_all(&self.pool)
         .await?;

        Ok(rows
            .into_iter()
            .map(|row| FileUploadPart {
                part_number: row.get("part_number"),
                etag: row.get("etag"),
            })
            .collect())
    }

    async fn upsert_upload_part(
        &self,
        upload_id: &str,
        part_number: i32,
        etag: &str,
    ) -> AppResult<()> {
        sqlx::query(
             "INSERT INTO file_upload_parts (upload_id, part_number, etag) VALUES ($1, $2, $3) ON CONFLICT (upload_id, part_number) DO UPDATE SET etag = EXCLUDED.etag",
         )
         .bind(upload_id)
         .bind(part_number)
         .bind(etag)
         .execute(&self.pool)
         .await?;

        sqlx::query("UPDATE file_uploads SET updated_at = NOW() WHERE upload_id = $1")
            .bind(upload_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_upload(&self, upload_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM file_uploads WHERE upload_id = $1")
            .bind(upload_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
