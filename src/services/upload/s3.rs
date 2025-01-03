// services/upload/s3.rs
use super::ImageUploader;
use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_s3::{config::Credentials, Client};
use uuid::Uuid;

#[derive(Clone)]
pub struct S3Uploader {
    client: Client,
    bucket: String,
}

impl S3Uploader {
    pub async fn new(
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        region: &str,
    ) -> anyhow::Result<Self> {
        let credentials = Credentials::new(access_key, secret_key, None, None, "static");

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_owned()))
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket: bucket.to_owned(),
        })
    }
}

#[async_trait]
impl ImageUploader for S3Uploader {
    async fn upload(
        &self,
        image_data: &[u8],
        format: &str,
        folder: &str,
    ) -> anyhow::Result<String> {
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

    async fn delete(&self, file_id: &str) -> anyhow::Result<bool> {
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
