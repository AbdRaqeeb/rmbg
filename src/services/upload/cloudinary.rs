use super::ImageUploader;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use cloudinary::upload::{OptionalParameters, Source, Upload, result::UploadResult};
use std::collections::BTreeSet;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Error;

#[derive(Clone)]
pub struct CloudinaryUploader {
    cloudinary: Arc<Upload>,
}

impl CloudinaryUploader {
    pub fn new(cloud_name: &str, api_key: &str, api_secret: &str) -> Self {
        Self {
            cloudinary: Arc::new(Upload::new(
                api_key.to_string(),
                cloud_name.to_string(),
                api_secret.to_string(),
            ))
        }
    }
}

#[async_trait]
impl ImageUploader for CloudinaryUploader {
    async fn upload(&self, image_data: &[u8], format: &str, folder: &str) -> Result<String, Error> {
        let unique_filename = format!("{}", Uuid::new_v4());
        let base64_data = BASE64.encode(image_data);
        let data_url = format!("data:image/{};base64,{}", format, base64_data);

        let options = BTreeSet::from([
            OptionalParameters::PublicId(unique_filename),
            OptionalParameters::Folder(folder.to_string()),
        ]);

        let response = self
            .cloudinary
            .image(Source::DataUrl(data_url), &options)
            .await
            .map_err(|e| anyhow::anyhow!("Cloudinary upload failed: {}", e))?;

        match response {
            UploadResult::Response(response) => Ok(response.secure_url),
            UploadResult::ResponseWithImageMetadata(response) => Ok(response.secure_url),
            UploadResult::Error(error) => {
                log::error!("Upload failed: {:?}", error);
                Err(anyhow::anyhow!("Upload failed"))
            }
        }
    }

    async fn delete(&self, public_id: &str) -> Result<bool, Error> {
        let response = self
            .cloudinary
            .destroy(public_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete image: {}", e))?;

        Ok(response.result == "ok")
    }
}
