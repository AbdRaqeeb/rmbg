use super::ImageUploader;
use async_trait::async_trait;
use anyhow::Result;
use minio_rsc::client::BucketArgs;
use minio_rsc::provider::StaticProvider;
use minio_rsc::Minio;
use std::sync::Arc;
use bytes::Bytes;
use uuid::Uuid;

#[derive(Clone)]
pub struct MinioUploader {
    client: Arc<Minio>,
    bucket: String,
    endpoint: String,
}

impl MinioUploader {
    pub fn new(
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        endpoint: &str,
        secure: bool,
    ) -> Result<Self> {
        // Create MinIO provider with credentials
        let provider = StaticProvider::new(access_key, secret_key, None);

        // Build MinIO client
        let client = Minio::builder()
            .endpoint(endpoint)
            .provider(provider)
            .secure(secure)
            .build()?;

        // Ensure bucket exists
        let bucket_exists = tokio::runtime::Runtime::new()?
            .block_on(async {
                let (buckets, _) = client.list_buckets().await?;
                Ok::<_, anyhow::Error>(buckets.iter().any(|b| b.name == bucket))
            })?;

        if !bucket_exists {
            tokio::runtime::Runtime::new()?
                .block_on(async {
                    client.make_bucket(BucketArgs::new(bucket), false).await
                })?;
        }

        Ok(Self {
            client: Arc::new(client),
            bucket: bucket.to_owned(),
            endpoint: if endpoint.ends_with('/') {
                endpoint.to_owned()
            } else {
                format!("{}/", endpoint)
            },
        })
    }

    fn build_url(&self, key: &str) -> String {
        format!("{}{}/{}", self.endpoint, self.bucket, key)
    }
}

#[async_trait]
impl ImageUploader for MinioUploader {
    async fn upload(&self, image_data: &[u8], format: &str, folder: &str) -> Result<String> {
        let key = format!("{}/{}.{}", folder, Uuid::new_v4(), format);

        self.client
            .put_object(&self.bucket, &key, Bytes::from(image_data.to_vec()))
            .await
            .map_err(|e| anyhow::anyhow!("MinIO upload failed: {}", e))?;

        Ok(self.build_url(&key))
    }

    async fn delete(&self, file_id: &str) -> Result<bool> {
        self.client
            .remove_object(&self.bucket, file_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete from MinIO: {}", e))?;

        Ok(true)
    }
}
