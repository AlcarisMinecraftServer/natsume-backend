use async_trait::async_trait;
use axum::body::Bytes;
use chrono::Utc;
use domain::files::FileMetadata;
use infrastructure::repositorys::file::FileRepository;
use shared::error::AppResult;
use std::env;
use uuid::Uuid;

use s3::creds::Credentials;
use s3::{Bucket, Region};

pub struct FileUsecaseImpl<R: FileRepository + Send + Sync> {
    pub repo: R,
}

impl<R: FileRepository + Send + Sync> FileUsecaseImpl<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
pub trait FileUsecase: Send + Sync {
    async fn upload_file(
        &self,
        user_id: &str,
        filename: &str,
        content_type: &str,
        data: Bytes,
    ) -> AppResult<FileMetadata>;

    async fn get_file(&self, file_id: &str) -> AppResult<FileMetadata>;

    async fn list_files(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>>;

    async fn delete_file(&self, file_id: &str) -> AppResult<()>;
}

#[async_trait]
impl<R: FileRepository + Send + Sync> FileUsecase for FileUsecaseImpl<R> {
    async fn upload_file(
        &self,
        user_id: &str,
        filename: &str,
        content_type: &str,
        data: Bytes,
    ) -> AppResult<FileMetadata> {
        let file_id = Uuid::new_v4().to_string();
        let key = format!("files/{}/{}/{}", user_id, file_id, filename);

        let bucket = env::var("R2_BUCKET_NAME")?;
        let endpoint = env::var("R2_ENDPOINT")?;
        let access_key = env::var("R2_ACCESS_KEY_ID")?;
        let secret_key = env::var("R2_SECRET_ACCESS_KEY")?;

        let region = Region::Custom {
            region: "auto".into(),
            endpoint,
        };

        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)?;

        let bucket = Bucket::new(&bucket, region, credentials)?.with_path_style();

        let response = bucket
            .put_object_with_content_type(&key, &data, content_type)
            .await?;
        let code = response.status_code();

        if code != 200 {
            return Err(anyhow::anyhow!("R2 upload failed (status code {})", code));
        }

        let metadata = FileMetadata {
            id: file_id.clone(),
            user_id: user_id.to_string(),
            filename: filename.to_string(),
            content_type: content_type.to_string(),
            size: data.len() as i64,
            uploaded_at: Utc::now().naive_utc(),
            url: Some(format!(
                "https://cdn.alcaris.net/files/{}/{}/{}",
                user_id, file_id, filename
            )),
        };

        self.repo.insert_metadata(&metadata).await?;
        Ok(metadata)
    }

    async fn get_file(&self, file_id: &str) -> AppResult<FileMetadata> {
        self.repo.find_metadata(file_id).await
    }

    async fn list_files(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>> {
        self.repo.list_metadata(user_id).await
    }

    async fn delete_file(&self, file_id: &str) -> AppResult<()> {
        let metadata = self.repo.find_metadata(file_id).await?;
        let key = format!(
            "files/{}/{}/{}",
            metadata.user_id, metadata.id, metadata.filename
        );

        let bucket_name = env::var("R2_BUCKET_NAME")?;
        let endpoint = env::var("R2_ENDPOINT")?;
        let access_key = env::var("R2_ACCESS_KEY_ID")?;
        let secret_key = env::var("R2_SECRET_ACCESS_KEY")?;

        let region = Region::Custom {
            region: "auto".into(),
            endpoint,
        };
        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)?;
        let bucket = Bucket::new(&bucket_name, region, credentials)?.with_path_style();

        let response = bucket.delete_object(&key).await?;
        let code = response.status_code();
        if code != 204 && code != 200 {
            return Err(anyhow::anyhow!(
                "Failed to delete file from R2 (status code {})",
                code
            ));
        }

        self.repo.delete_metadata(file_id).await
    }
}
