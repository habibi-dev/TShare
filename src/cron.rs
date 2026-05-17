use crate::core::state::AppState;
use crate::features::storage::cleanup::register_cleanup_job;

pub struct Cron;

impl Cron {
    pub async fn start(app_state: AppState) {
        if app_state.file_upload.upload_enabled {
            let interval = app_state.file_upload.cleanup_interval_secs;
            register_cleanup_job(interval).start();
        }
    }
}
