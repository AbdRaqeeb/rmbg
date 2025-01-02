use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use std::sync::Arc;

mod config;
mod error;
mod routes;
mod services;

use config::AppConfig;

use crate::services::onnx::onnx_session;
use services::upload::CloudinaryUploader;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file first
    dotenv().ok();

    // Initialize configuration
    let config = Arc::new(AppConfig::from_env().expect("Failed to load configuration"));
    let bind_host = config.server.host.clone();
    let bind_port = config.server.port;

    // Setup logging based on LOG_LEVEL
    setup_logging();

    // Create and configure temporary upload directory
    let tmp_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./tmp".to_string());
    log::info!("Creating temporary upload directory: {}", tmp_dir);
    std::fs::create_dir_all(&tmp_dir)?;

    // Initialize ONNX session
    let session = Arc::new(onnx_session(&config.model.path).expect("Failed to load ONNX model"));
    log::info!("Loaded ONNX model: {:?}", config.model.size);

    // Initialize Cloudinary uploader
    let uploader = CloudinaryUploader::new(
        &config.cloudinary.cloud_name,
        &config.cloudinary.api_key,
        &config.cloudinary.api_secret,
    );

    log::info!("Starting server at {}:{}", bind_host, bind_port);

    // Start HTTP server
    HttpServer::new(move || {
        // Configure multipart/form-data handling
        let temp_config = TempFileConfig::default().directory(&tmp_dir);

        App::new()
            // Add CORS middleware
            .wrap(
                middleware::DefaultHeaders::new()
                    .add((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
                    .add((header::ACCESS_CONTROL_ALLOW_METHODS, "POST, GET, OPTIONS"))
                    .add((
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        "Content-Type, Authorization, X-Auth-Token",
                    )),
            )
            // Configure application data
            .app_data(web::Data::from(Arc::clone(&config)))
            .app_data(web::Data::from(Arc::clone(&session)))
            .app_data(web::Data::new(uploader.clone()))
            .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024)) // 10MB JSON payload limit
            .app_data(web::FormConfig::default().limit(32 * 1024 * 1024)) // 32MB form payload limit
            .app_data(temp_config)
            // Mount routes with explicit path configuration
            .service(web::scope("/api").service(routes::image::process_and_upload))
            .service(routes::health::index)
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(Logger::default())
    })
    .bind((bind_host, bind_port))?
    .run()
    .await
}

fn setup_logging() {
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();

    // Convert string log level to log::LevelFilter
    let level = match log_level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => {
            eprintln!(
                "Invalid LOG_LEVEL value: '{}', defaulting to 'info'",
                log_level
            );
            log::LevelFilter::Info
        }
    };

    env_logger::Builder::new().filter_level(level).init();
}
