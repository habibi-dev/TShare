use crate::core::config::Config;
use crate::features::storage::FileUploadConfig;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use std::sync::{Arc, OnceLock};
use std::time::Instant;
pub static APP_STATE: OnceLock<AppState> = OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    pub _db: DatabaseConnection,
    pub redis: Arc<ConnectionManager>,
    pub config: Config,
    pub file_upload: FileUploadConfig,
    pub uptime: Instant,
}

pub struct State;

impl State {
    pub fn init(
        db: DatabaseConnection,
        config: Config,
        redis: Arc<ConnectionManager>,
        file_upload: FileUploadConfig,
    ) {
        APP_STATE
            .set(AppState {
                _db: db,
                redis,
                config,
                file_upload,
                uptime: Instant::now(),
            })
            .ok();
    }
}
