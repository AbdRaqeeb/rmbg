use std::collections::HashMap;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::services::upload::{DynImageUploader, UploaderType};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub session: Arc<ort::Session>,
    pub uploaders: Arc<HashMap<UploaderType, DynImageUploader>>,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        session: Arc<ort::Session>,
        uploaders: HashMap<UploaderType, DynImageUploader>,
    ) -> Self {
        Self {
            config,
            session,
            uploaders: Arc::new(uploaders),
        }
    }
}
