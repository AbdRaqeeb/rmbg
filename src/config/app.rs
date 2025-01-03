use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use std::env;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub cloudinary: CloudinaryConfig,
    pub minio: MinioConfig,
    pub model: ModelConfig,
    pub s3: S3Config,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct CloudinaryConfig {
    pub cloud_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub upload_preset: String,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

#[derive(Debug, Clone)]
pub struct MinioConfig {
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub endpoint: String,
    pub secure: bool,
    pub region: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
}

impl FromStr for ModelSize {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "small" => Ok(ModelSize::Small),
            "medium" => Ok(ModelSize::Medium),
            "large" => Ok(ModelSize::Large),
            _ => Err(anyhow!(
                "Invalid model size. Valid values are: small, medium, large"
            )),
        }
    }
}

impl ModelSize {
    pub fn get_model_path(&self) -> String {
        let model_name = match self {
            ModelSize::Small => "small",
            ModelSize::Medium => "medium",
            ModelSize::Large => "large",
        };
        format!("models/{}.onnx", model_name)
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub size: ModelSize,
    pub path: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let model_size = ModelSize::from_str(
            &env::var("ONNX_MODEL_SIZE").unwrap_or_else(|_| "medium".to_string()),
        )?;

        let model_config = ModelConfig {
            size: model_size,
            path: model_size.get_model_path(),
        };

        Ok(Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
            cloudinary: CloudinaryConfig {
                cloud_name: env::var("CLOUDINARY_CLOUD_NAME")?,
                api_key: env::var("CLOUDINARY_API_KEY")?,
                api_secret: env::var("CLOUDINARY_API_SECRET")?,
                upload_preset: env::var("CLOUDINARY_UPLOAD_PRESET")?,
            },
            model: model_config,
            s3: S3Config {
                access_key: env::var("AWS_ACCESS_KEY_ID").unwrap_or("".to_string()),
                secret_key: env::var("AWS_SECRET_ACCESS_KEY").unwrap_or("".to_string()),
                bucket: env::var("S3_BUCKET").unwrap_or("".to_string()),
                region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            },
            minio: MinioConfig {
                access_key: env::var("MINIO_ACCESS_KEY").unwrap_or("".to_string()),
                secret_key: env::var("MINIO_SECRET_KEY").unwrap_or("".to_string()),
                bucket: env::var("MINIO_BUCKET").unwrap_or("".to_string()),
                endpoint: env::var("MINIO_ENDPOINT").unwrap_or("".to_string()),
                secure: env::var("MINIO_SECURE")
                    .map(|v| v.parse::<bool>().unwrap_or(true))
                    .unwrap_or(true),
                region: env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            },
        })
    }
}
