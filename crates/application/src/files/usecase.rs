use async_trait::async_trait;
use chrono::Utc;
use domain::files::{FileMetadata, FileUploadPart, FileUploadSession};
use infrastructure::repositorys::file::FileRepository;
use shared::error::AppResult;
use std::{collections::HashMap, env};
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
    async fn create_upload(
        &self,
        user_id: &str,
        filename: &str,
        content_type: &str,
        size: i64,
    ) -> AppResult<FileUploadSession>;

    async fn get_upload(&self, upload_id: &str) -> AppResult<(FileUploadSession, Vec<FileUploadPart>)>;

    async fn get_part_upload_url(&self, upload_id: &str, part_number: i32) -> AppResult<String>;

    async fn register_part(&self, upload_id: &str, part_number: i32, etag: &str) -> AppResult<()>;

    async fn complete_upload(
        &self,
        upload_id: &str,
        uploader_username: Option<String>,
        uploader_global_name: Option<String>,
        uploader_avatar_url: Option<String>,
    ) -> AppResult<FileMetadata>;

    async fn abort_upload(&self, upload_id: &str) -> AppResult<()>;

    async fn get_file_by_id(&self, file_id: &str) -> AppResult<FileMetadata>;

    async fn find_all_files(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>>;

    async fn delete_file(&self, file_id: &str) -> AppResult<()>;
}

#[async_trait]
impl<R: FileRepository + Send + Sync> FileUsecase for FileUsecaseImpl<R> {
    async fn create_upload(
        &self,
        user_id: &str,
        filename: &str,
        content_type: &str,
        size: i64,
    ) -> AppResult<FileUploadSession> {
        const DEFAULT_PART_SIZE: i64 = 16 * 1024 * 1024;

        let filename = sanitize_filename(filename);
        let file_id = Uuid::new_v4().to_string();
        let key = format!("files/{}/{}/{}", user_id, file_id, filename);

        let bucket = make_bucket().await?;
        let initiated = bucket
            .initiate_multipart_upload(&key, content_type)
            .await?;

        let upload = FileUploadSession {
            upload_id: initiated.upload_id,
            file_id,
            user_id: user_id.to_string(),
            key,
            filename,
            content_type: content_type.to_string(),
            size,
            part_size: DEFAULT_PART_SIZE,
        };

        self.repo.create_upload(&upload).await?;
        Ok(upload)
    }

    async fn get_upload(&self, upload_id: &str) -> AppResult<(FileUploadSession, Vec<FileUploadPart>)> {
        let upload = self.repo.find_upload(upload_id).await?;
        let parts = self.repo.list_upload_parts(upload_id).await?;
        Ok((upload, parts))
    }

    async fn get_part_upload_url(&self, upload_id: &str, part_number: i32) -> AppResult<String> {
        const PRESIGN_EXPIRES_SECS: u32 = 60 * 10;

        let upload = self.repo.find_upload(upload_id).await?;
        let bucket = make_bucket().await?;

        let mut queries = HashMap::new();
        queries.insert("partNumber".to_string(), part_number.to_string());
        queries.insert("uploadId".to_string(), upload.upload_id.clone());

        let url = bucket
            .presign_put(&upload.key, PRESIGN_EXPIRES_SECS, None, Some(queries))
            .await?;

        Ok(url)
    }

    async fn register_part(&self, upload_id: &str, part_number: i32, etag: &str) -> AppResult<()> {
        self.repo
            .upsert_upload_part(upload_id, part_number, etag)
            .await
    }

    async fn complete_upload(
        &self,
        upload_id: &str,
        uploader_username: Option<String>,
        uploader_global_name: Option<String>,
        uploader_avatar_url: Option<String>,
    ) -> AppResult<FileMetadata> {
        let (upload, mut parts) = self.get_upload(upload_id).await?;

        parts.sort_by_key(|p| p.part_number);
        let parts: Vec<s3::serde_types::Part> = parts
            .into_iter()
            .map(|p| s3::serde_types::Part {
                part_number: p.part_number.max(1) as u32,
                etag: p.etag,
            })
            .collect();

        let bucket = make_bucket().await?;
        let response = bucket
            .complete_multipart_upload(&upload.key, &upload.upload_id, parts)
            .await?;
        let code = response.status_code();

        if code != 200 {
            return Err(anyhow::anyhow!(
                "R2 multipart completion failed (status code {})",
                code
            ));
        }

        let metadata = FileMetadata {
            id: upload.file_id.clone(),
            user_id: upload.user_id.clone(),
            filename: upload.filename.clone(),
            content_type: upload.content_type.clone(),
            size: upload.size,
            uploaded_at: Utc::now().naive_utc(),
            url: Some(format!(
                "https://cdn.alcaris.net/files/{}/{}/{}",
                upload.user_id, upload.file_id, upload.filename
            )),
            uploader_username,
            uploader_global_name,
            uploader_avatar_url,
        };

        self.repo.insert_metadata(&metadata).await?;
        self.repo.delete_upload(upload_id).await?;
        Ok(metadata)
    }

    async fn abort_upload(&self, upload_id: &str) -> AppResult<()> {
        let upload = self.repo.find_upload(upload_id).await?;
        let bucket = make_bucket().await?;
        bucket.abort_upload(&upload.key, &upload.upload_id).await?;
        self.repo.delete_upload(upload_id).await?;
        Ok(())
    }

    async fn get_file_by_id(&self, file_id: &str) -> AppResult<FileMetadata> {
        self.repo.find_metadata(file_id).await
    }

    async fn find_all_files(&self, user_id: Option<String>) -> AppResult<Vec<FileMetadata>> {
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
        let secret_key = env::var("R2_ACCESS_KEY_SECRET")?;

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

fn sanitize_filename(filename: &str) -> String {
    let filename = filename.trim();
    if filename.is_empty() {
        return "file".to_string();
    }

    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' => '_',
            '\u{0000}'..='\u{001f}' => '_',
            _ => c,
        })
        .collect()
}

async fn make_bucket() -> AppResult<Box<Bucket>> {
    let bucket_name = env::var("R2_BUCKET_NAME")?;
    let endpoint = env::var("R2_ENDPOINT")?;
    let access_key = env::var("R2_ACCESS_KEY_ID")?;
    let secret_key = env::var("R2_ACCESS_KEY_SECRET")?;

    let region = Region::Custom {
        region: "auto".into(),
        endpoint,
    };

    let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)?;
    Ok(Bucket::new(&bucket_name, region, credentials)?.with_path_style())
}
