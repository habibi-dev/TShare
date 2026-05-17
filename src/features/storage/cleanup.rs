use crate::core::cron_manager::{boxed, CronDefinition, CronManager};
use crate::features::storage::service::StorageService;
use crate::utility::state::app_state;
use chrono::Utc;
use redis::AsyncCommands;
use std::time::Duration;
use tracing::{error, info};

pub const CLEANUP_ZSET_KEY: &str = "file_cleanup";

pub fn register_cleanup_job(interval_secs: u64) -> CronManager {
    let interval = Duration::from_secs(interval_secs.max(10));
    CronManager::new(vec![CronDefinition {
        name: "file_cleanup",
        interval,
        tasks: vec![boxed(run_file_cleanup)],
    }])
}

async fn run_file_cleanup() {
    let state = app_state();
    if !state.file_upload.upload_enabled {
        return;
    }

    let storage = match StorageService::from_state() {
        Ok(s) => s,
        Err(e) => {
            error!(target: "system", "File cleanup: storage unavailable: {}", e);
            return;
        }
    };

    let now = Utc::now().timestamp() as f64;
    let mut redis = state.redis.as_ref().clone();

    let expired: Vec<String> = match redis.zrangebyscore(CLEANUP_ZSET_KEY, 0.0, now).await {
        Ok(keys) => keys,
        Err(e) => {
            error!(target: "system", "File cleanup ZRANGE failed: {}", e);
            return;
        }
    };

    if expired.is_empty() {
        return;
    }

    for stored_name in &expired {
        if let Err(e) = storage.delete_stored(stored_name).await {
            error!(
                target: "system",
                "File cleanup delete failed for {}: {}",
                stored_name,
                e
            );
        }
        let _: Result<(), _> = redis.zrem(CLEANUP_ZSET_KEY, stored_name).await;
    }

    info!(
        target: "system",
        "File cleanup removed {} expired file(s)",
        expired.len()
    );
}
