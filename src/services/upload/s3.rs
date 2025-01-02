use anyhow::Error;
use super::ImageUploader;
use async_trait::async_trait;
use aws_sdk_s3::{Client};
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, Region};
use uuid::Uuid;

#[derive(Clone)]
pub struct S3Uploader {
    client: Client,
    bucket: String,
}

impl S3Uploader {
    pub fn new(access_key: &str, secret_key: &str, bucket: &str, region: &str) -> Self {
        let region_provider = RegionProviderChain::first_try(Region::new(region.to_owned()));

        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .behavior_version(BehaviorVersion::latest())
            .region(region_provider)
            .credentials_provider(
                aws_config::default_provider::credentials::DefaultCredentialsChain::builder()
                    .build()
            )
            .load()
            .expect("Failed to load AWS config");

        let client = Client::new(&config);

        Self {
            client,
            bucket: bucket.to_owned(),
        }
    }
}

#[async_trait]
impl ImageUploader for S3Uploader {
    async fn upload(&self, image_data: &[u8], format: &str, folder: &str) -> Result<String, Error> {
        let key = format!("{}/{}.{}", folder, Uuid::new_v4(), format);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(image_data.to_vec().into())
            .content_type(format!("image/{}", format))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("S3 upload failed: {}", e))?;

        Ok(format!("https://{}.s3.amazonaws.com/{}", self.bucket, key))
    }

    async fn delete(&self, file_id: &str) -> Result<bool, Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(file_id)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete from S3: {}", e))?;

        Ok(true)
    }
}
