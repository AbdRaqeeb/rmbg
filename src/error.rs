use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to process image: {0}")]
    ImageProcessing(String),

    #[error("Failed to upload to Cloudinary: {0}")]
    CloudinaryUpload(String),

    #[error("Invalid file format")]
    InvalidFileFormat,

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        log::error!("Error: {:?}", self);

        match self {
            AppError::InvalidFileFormat => HttpResponse::BadRequest().json(json!({
                "error": "Invalid file format"
            })),
            AppError::ImageProcessing(err) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to process image: {}", err)
            })),
            AppError::CloudinaryUpload(err) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to upload to Cloudinary: {}", err)
            })),
            AppError::InternalError(err) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Internal server error: {}", err)
            })),
        }
    }
}
