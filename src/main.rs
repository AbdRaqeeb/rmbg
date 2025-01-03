use dotenvy::dotenv;
use std::sync::Arc;

mod config;
mod error;
mod routes;
mod server;
mod services;
mod utils;

use config::AppConfig;
use server::{create_server, setup::initialize_uploaders, AppState};
use services::onnx::onnx_session;
use utils::logging::setup_logging;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize configuration
    let config = Arc::new(AppConfig::from_env().expect("Failed to load configuration"));

    // Setup logging
    setup_logging();

    // Setup temporary directory
    let tmp_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./tmp".to_string());
    log::info!("Creating temporary upload directory: {}", tmp_dir);
    std::fs::create_dir_all(&tmp_dir)?;

    // Initialize ONNX session
    let session = Arc::new(onnx_session(&config.model.path).expect("Failed to load ONNX model"));
    log::info!("Loaded ONNX model: {:?}", config.model.size);

    // Initialize uploaders
    let uploaders = initialize_uploaders(&config)
        .await
        .expect("Failed to initialize uploaders");

    // Create application state
    let app_state = AppState::new(Arc::clone(&config), session, uploaders);

    log::info!(
        "Starting server at {}:{}",
        config.server.host,
        config.server.port
    );

    // Create and start server
    let server = create_server(
        app_state,
        tmp_dir,
        config.server.host.clone(),
        config.server.port,
    )?;

    server.await
}
