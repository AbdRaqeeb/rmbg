mod cloudinary;
mod s3;
mod minio;

use async_trait::async_trait;
use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;
use crate::services::upload::cloudinary::CloudinaryUploader;
use crate::services::upload::minio::MinioUploader;
use crate::services::upload::s3::S3Uploader;

#[derive(Debug, Clone, Deserialize)]
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
    pub fn create_uploader(uploader_type: UploaderType, config: &crate::config::AppConfig) -> DynImageUploader {
        match uploader_type {
            UploaderType::Cloudinary => Arc::new(CloudinaryUploader::new(
                &config.cloudinary.cloud_name,
                &config.cloudinary.api_key,
                &config.cloudinary.api_secret,
            )),
            UploaderType::S3 => Arc::new(S3Uploader::new(
                &config.s3.access_key,
                &config.s3.secret_key,
                &config.s3.bucket,
                &config.s3.region,
            )),
            UploaderType::Minio => Arc::new(MinioUploader::new(
                &config.minio.access_key,
                &config.minio.secret_key,
                &config.minio.bucket,
                &config.minio.endpoint,
                config.minio.secure,
            ).expect("Failed to create MinIO uploader")),
        }
    }
}
