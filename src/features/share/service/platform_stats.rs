use crate::features::setting::service::setting_service::SettingService;
use crate::utility::format::{format_bytes, format_count};
use lazy_static::lazy_static;
use sea_orm::DbErr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(600);

const KEY_USED: &str = "used";
const KEY_TEXT: &str = "stats_text";
const KEY_FILES: &str = "stats_files";
const KEY_BYTES_UPLOADED: &str = "stats_bytes_uploaded";
const KEY_BYTES_DOWNLOADED: &str = "stats_bytes_downloaded";

struct CachedStats {
    stats: PlatformStats,
    last_updated: Instant,
}

lazy_static! {
    static ref STATS_CACHE: RwLock<Option<CachedStats>> = RwLock::new(None);
}

#[derive(Debug, Clone)]
pub struct PlatformStats {
    pub total_shares: i64,
    pub text_shares: i64,
    pub file_shares: i64,
    pub bytes_uploaded: u64,
    pub bytes_downloaded: u64,
}

#[derive(Debug, Clone)]
pub struct PlatformStatsDisplay {
    pub total_shares: String,
    pub text_shares: String,
    pub file_shares: String,
    pub uploaded_size: String,
    pub downloaded_size: String,
}

impl PlatformStatsDisplay {
    pub fn from_stats(stats: &PlatformStats) -> Self {
        Self {
            total_shares: format_count(stats.total_shares),
            text_shares: format_count(stats.text_shares),
            file_shares: format_count(stats.file_shares),
            uploaded_size: format_bytes(stats.bytes_uploaded),
            downloaded_size: format_bytes(stats.bytes_downloaded),
        }
    }
}

pub async fn get_platform_stats() -> Result<PlatformStats, DbErr> {
    {
        let cache = STATS_CACHE.read().unwrap();
        if let Some(cached) = cache.as_ref()
            && cached.last_updated.elapsed() < CACHE_TTL
        {
            return Ok(cached.stats.clone());
        }
    }

    let stats = fetch_stats_from_db().await?;

    {
        let mut cache = STATS_CACHE.write().unwrap();
        *cache = Some(CachedStats {
            stats: stats.clone(),
            last_updated: Instant::now(),
        });
    }

    Ok(stats)
}

pub async fn get_platform_stats_display() -> PlatformStatsDisplay {
    let stats = get_platform_stats().await.unwrap_or_default();
    PlatformStatsDisplay::from_stats(&stats)
}

pub async fn record_share_created(has_text: bool, file_size: Option<u64>) -> Result<(), DbErr> {
    increment_counter(KEY_USED, 1).await?;
    if has_text {
        increment_counter(KEY_TEXT, 1).await?;
    }
    if let Some(size) = file_size {
        if size > 0 {
            increment_counter(KEY_FILES, 1).await?;
            add_bytes(KEY_BYTES_UPLOADED, size).await?;
        }
    }
    invalidate_stats_cache();
    Ok(())
}

pub async fn record_file_downloaded(bytes: u64) -> Result<(), DbErr> {
    if bytes > 0 {
        add_bytes(KEY_BYTES_DOWNLOADED, bytes).await?;
        invalidate_stats_cache();
    }
    Ok(())
}

pub fn invalidate_stats_cache() {
    let mut cache = STATS_CACHE.write().unwrap();
    *cache = None;
}

impl Default for PlatformStats {
    fn default() -> Self {
        Self {
            total_shares: 0,
            text_shares: 0,
            file_shares: 0,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
        }
    }
}

async fn fetch_stats_from_db() -> Result<PlatformStats, DbErr> {
    Ok(PlatformStats {
        total_shares: get_i64_setting(KEY_USED).await?,
        text_shares: get_i64_setting(KEY_TEXT).await?,
        file_shares: get_i64_setting(KEY_FILES).await?,
        bytes_uploaded: get_u64_setting(KEY_BYTES_UPLOADED).await?,
        bytes_downloaded: get_u64_setting(KEY_BYTES_DOWNLOADED).await?,
    })
}

async fn get_i64_setting(key: &str) -> Result<i64, DbErr> {
    Ok(SettingService::get_by_key(key)
        .await?
        .and_then(|s| s.meta_value)
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0))
}

async fn get_u64_setting(key: &str) -> Result<u64, DbErr> {
    Ok(SettingService::get_by_key(key)
        .await?
        .and_then(|s| s.meta_value)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0))
}

async fn increment_counter(key: &str, delta: i64) -> Result<(), DbErr> {
    let current = get_i64_setting(key).await?;
    SettingService::upsert(key.to_string(), Some((current + delta).to_string())).await?;
    Ok(())
}

async fn add_bytes(key: &str, delta: u64) -> Result<(), DbErr> {
    let current = get_u64_setting(key).await?;
    SettingService::upsert(key.to_string(), Some((current + delta).to_string())).await?;
    Ok(())
}
