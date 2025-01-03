// server/setup.rs
use std::collections::HashMap;

use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{middleware, web, App, HttpServer};

use super::state::AppState;
use crate::config::AppConfig;
use crate::routes;
use crate::services::upload::{DynImageUploader, UploaderFactory, UploaderType};

// server/setup.rs

pub async fn initialize_uploaders(
    config: &AppConfig,
) -> anyhow::Result<HashMap<UploaderType, DynImageUploader>> {
    let mut uploaders = HashMap::new();

    // Initialize each uploader type
    let uploader_types = vec![
        UploaderType::Cloudinary,
        UploaderType::S3,
        UploaderType::Minio,
    ];

    for uploader_type in uploader_types {
        let uploader = UploaderFactory::create_uploader(uploader_type.clone(), config).await?;
        uploaders.insert(uploader_type, uploader);
    }

    Ok(uploaders)
}

fn configure_cors() -> middleware::DefaultHeaders {
    middleware::DefaultHeaders::new()
        .add((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .add((header::ACCESS_CONTROL_ALLOW_METHODS, "POST, GET, OPTIONS"))
        .add((
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Content-Type, Authorization, X-Auth-Token",
        ))
}

fn configure_temp_files(tmp_dir: &str) -> TempFileConfig {
    TempFileConfig::default().directory(tmp_dir)
}

pub fn create_server(
    app_state: AppState,
    tmp_dir: String,
    bind_host: String,
    bind_port: u16,
) -> std::io::Result<actix_web::dev::Server> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(configure_cors())
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024))
            .app_data(web::FormConfig::default().limit(32 * 1024 * 1024))
            .app_data(configure_temp_files(&tmp_dir))
            .service(web::scope("/api").service(routes::image::process_and_upload))
            .service(routes::health::index)
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(Logger::default())
    })
    .bind((bind_host, bind_port))?
    .run();

    Ok(server)
}
