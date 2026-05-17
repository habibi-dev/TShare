use crate::features::share::service::platform_stats::{get_platform_stats, invalidate_stats_cache};
use sea_orm::DbErr;

pub async fn get_used_count() -> Result<i64, DbErr> {
    Ok(get_platform_stats().await?.total_shares)
}

pub async fn invalidate_used_cache() {
    invalidate_stats_cache();
}
