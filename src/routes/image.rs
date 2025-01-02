use crate::config::AppConfig;
use crate::error::AppError;
use crate::services::{
    image::{find_alpha_bounds, process_image},
    upload::{UploaderType, UploaderFactory, ImageUploader},
};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{post, web, HttpResponse};
use futures::future::try_join_all;
use image::imageops;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
struct ProcessQuery {
    crop: Option<bool>,
    #[serde(default)]
    upload: UploaderType,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "files", limit = "32MiB")]
    files: Vec<TempFile>,
}

#[derive(Debug, Serialize)]
struct ProcessedImageResult {
    secure_url: String,
}

#[post("/process")]
pub async fn process_and_upload(
    MultipartForm(form): MultipartForm<UploadForm>,
    session: web::Data<Arc<ort::Session>>,
    config: web::Data<AppConfig>,
    query: web::Query<ProcessQuery>,
) -> Result<HttpResponse, AppError> {
    // Create uploader based on query parameter
    let uploader = UploaderFactory::create_uploader(query.upload.clone(), &config);

    log::info!("Using uploader: {:?}", query.upload);
    log::info!("Received form data with {} files", form.files.len());

    if form.files.is_empty() {
        log::warn!("No files received");
        return Err(AppError::InvalidFileFormat);
    }

    // Process all files concurrently
    let processing_futures: Vec<_> = form.files
        .into_iter()
        .map(|file| {
            let pr_session = Arc::clone(&session);
            let pr_uploader = Arc::clone(&uploader);
            let pr_config = config.clone();
            let should_crop = query.crop.unwrap_or(false);

            async move {
                log::info!("Processing file: {:?}", file.file_name);

                let image_data = tokio::fs::read(file.file.path())
                    .await
                    .map_err(|e| {
                        log::error!("Error reading uploaded file: {}", e);
                        AppError::InternalError(e.to_string())
                    })?;

                if image_data.is_empty() {
                    log::warn!("Empty file received");
                    return Err(AppError::InvalidFileFormat);
                }

                log::info!("File size: {} bytes", image_data.len());

                process_single_image(
                    image_data,
                    &pr_session,
                    &*pr_uploader,
                    &pr_config,
                    should_crop
                ).await
            }
        })
        .collect();

    // Wait for all processing to complete
    let results = try_join_all(processing_futures).await?;

    log::info!("Successfully processed and uploaded {} images", results.len());

    Ok(HttpResponse::Ok().json(json!({
        "results": results
    })))
}

async fn process_single_image(
    image_data: Vec<u8>,
    session: &Arc<ort::Session>,
    uploader: &dyn ImageUploader,
    config: &AppConfig,
    should_crop: bool,
) -> Result<ProcessedImageResult, AppError> {
    // Process image with ONNX model
    log::info!("Processing image with ONNX model");
    let processed = process_image(session, &image_data).await
        .map_err(|e| {
            log::error!("Image processing failed: {}", e);
            AppError::ImageProcessing(e.to_string())
        })?;

    // If no cropping requested, upload the processed image directly
    if !should_crop {
        let secure_url = upload_to_storage(uploader, &processed.data).await?;
        return Ok(ProcessedImageResult { secure_url });
    }

    // Handle cropping
    let img = image::load_from_memory(&processed.data)
        .map_err(|e| AppError::ImageProcessing(e.to_string()))?;
    let mut output_img = img.to_rgba8();

    // Find alpha bounds for cropping
    let bounds = find_alpha_bounds(&output_img);

    match bounds {
        Some((min_x, min_y, max_x, max_y)) => {
            // Crop the image
            let cropped_img = imageops::crop(
                &mut output_img,
                min_x,
                min_y,
                max_x - min_x + 1,
                max_y - min_y + 1
            ).to_image();

            // Convert cropped image to PNG format
            let mut buffer = std::io::Cursor::new(Vec::new());
            cropped_img
                .write_to(&mut buffer, image::ImageFormat::Png)
                .map_err(|e| AppError::ImageProcessing(e.to_string()))?;

            // Upload the final cropped image
            let secure_url = upload_to_storage(uploader, &buffer.into_inner()).await?;
            Ok(ProcessedImageResult { secure_url })
        },
        None => {
            // If no valid bounds found, upload the processed image without cropping
            let secure_url = upload_to_storage(uploader, &processed.data).await?;
            Ok(ProcessedImageResult { secure_url })
        }
    }
}

async fn upload_to_storage(
    uploader: &dyn ImageUploader,
    image_data: &[u8],
) -> Result<String, AppError> {
    log::info!("Uploading to storage service");
    uploader
        .upload(image_data, "png", "uploads")
        .await
        .map_err(|e| {
            log::error!("Upload failed: {}", e);
            AppError::CloudinaryUpload(e.to_string())
        })
}
