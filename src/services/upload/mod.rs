use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub(crate) mod cloudinary;
pub(crate) mod minio;
pub(crate) mod s3;

pub use cloudinary::CloudinaryUploader;
pub use minio::MinioUploader;
pub use s3::S3Uploader;

#[derive(Debug, Clone, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploaderType {
    Cloudinary,
    S3,
    Minio,
    // Add more uploader types here
}

impl Default for UploaderType {
    fn default() -> Self {
        Self::Cloudinary
    }
}

#[async_trait]
pub trait ImageUploader: Send + Sync + 'static {
    async fn upload(&self, image_data: &[u8], format: &str, folder: &str) -> Result<String>;
    async fn delete(&self, file_id: &str) -> Result<bool>;
}

pub type DynImageUploader = Arc<dyn ImageUploader>;

// Factory for creating uploaders
pub struct UploaderFactory;

impl UploaderFactory {
    pub async fn create_uploader(
        uploader_type: UploaderType,
        config: &crate::config::AppConfig,
    ) -> Result<DynImageUploader> {
        let uploader: DynImageUploader = match uploader_type {
            UploaderType::Cloudinary => {
                if !&config.cloudinary.cloud_name.is_empty() {
                    ()
                };
                Arc::new(CloudinaryUploader::new(
                    &config.cloudinary.cloud_name,
                    &config.cloudinary.api_key,
                    &config.cloudinary.api_secret,
                ))
            }
            UploaderType::S3 => {
                if !&config.s3.access_key.is_empty() {
                    ()
                };
                Arc::new(
                    S3Uploader::new(
                        &config.s3.access_key,
                        &config.s3.secret_key,
                        &config.s3.bucket,
                        &config.s3.region,
                    )
                    .await?,
                )
            }
            UploaderType::Minio => {
                if !&config.minio.access_key.is_empty() {
                    ()
                };
                Arc::new(
                    MinioUploader::new(
                        &config.minio.access_key,
                        &config.minio.secret_key,
                        &config.minio.bucket,
                        &config.minio.endpoint,
                        config.minio.secure,
                    )
                    .await?,
                )
            }
        };

        Ok(uploader)
    }
}
