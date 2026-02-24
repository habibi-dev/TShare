use crate::features::setting::service::setting_service::SettingService;
use lazy_static::lazy_static;
use sea_orm::DbErr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(600); // 10 minutes

struct CachedValue {
    value: i64,
    last_updated: Instant,
}

lazy_static! {
    static ref USED_CACHE: RwLock<Option<CachedValue>> = RwLock::new(None);
}

pub async fn get_used_count() -> Result<i64, DbErr> {
    // Try to read from cache first
    {
        let cache = USED_CACHE.read().unwrap();
        if let Some(cached) = cache.as_ref()
            && cached.last_updated.elapsed() < CACHE_TTL
        {
            return Ok(cached.value);
        }
    }

    // Cache miss or expired, fetch from database
    let count = fetch_used_from_db().await?;

    // Update cache
    {
        let mut cache = USED_CACHE.write().unwrap();
        *cache = Some(CachedValue {
            value: count,
            last_updated: Instant::now(),
        });
    }

    Ok(count)
}

async fn fetch_used_from_db() -> Result<i64, DbErr> {
    let setting = SettingService::get_by_key("used").await?;

    Ok(setting
        .and_then(|s| s.meta_value)
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0))
}

// Force refresh cache (call this after increment)
pub async fn invalidate_used_cache() {
    let mut cache = USED_CACHE.write().unwrap();
    *cache = None;
}
