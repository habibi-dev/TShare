use crate::features::share::service::share_retrieve::ShareError;
use crate::features::share::utility::generate_unique_key::key_prefix;
use crate::features::share::validation::share_form::ShareForm;
use crate::utility::state::app_state;
use axum::http::StatusCode;
use redis::AsyncCommands;
use std::time::Duration;
use tracing::error;
use tokio::time::sleep;

const LOCK_PREFIX: &str = "share:lock:";
const LOCK_TTL_SECS: u64 = 8;
const LOCK_SPIN_MS: u64 = 25;
const MAX_LOCK_ATTEMPTS: u32 = 80;

/// Atomically read-modify-write share JSON in Redis (distributed lock per share key).
pub async fn update_share_atomic<F>(key: &str, mutator: F) -> Result<ShareForm, ShareError>
where
    F: Fn(&mut ShareForm) -> Result<(), ShareError>,
{
    let redis_key = key_prefix(&key.to_string());
    let lock_key = format!("{LOCK_PREFIX}{key}");

    for attempt in 0..MAX_LOCK_ATTEMPTS {
        let state = app_state();
        let mut redis = state.redis.as_ref().clone();

        let lock_acquired: bool = redis::cmd("SET")
            .arg(&lock_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(LOCK_TTL_SECS)
            .query_async::<Option<String>>(&mut redis)
            .await
            .map(|v| v.is_some())
            .unwrap_or(false);

        if !lock_acquired {
            if attempt + 1 >= MAX_LOCK_ATTEMPTS {
                return Err(ShareError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "سرور شلوغ است. لطفاً دوباره تلاش کنید.".to_string(),
                ));
            }
            sleep(Duration::from_millis(LOCK_SPIN_MS)).await;
            continue;
        }

        let result = async {
            let json_string: String = redis.get(&redis_key).await.map_err(|e| {
                error!(target: "system", "Redis GET failed during atomic update: {}", e);
                ShareError::new(
                    StatusCode::NOT_FOUND,
                    "اشتراک گذاری یافت نشد یا منقضی شده است.".to_string(),
                )
            })?;

            if json_string.is_empty() {
                return Err(ShareError::new(
                    StatusCode::NOT_FOUND,
                    "اشتراک گذاری یافت نشد یا منقضی شده است.".to_string(),
                ));
            }

            let ttl: i64 = redis.ttl(&redis_key).await.map_err(|e| {
                error!(target: "system", "Redis TTL failed: {}", e);
                ShareError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "خطا در دریافت زمان انقضا".to_string(),
                )
            })?;

            if ttl <= 0 {
                return Err(ShareError::new(
                    StatusCode::GONE,
                    "اشتراک گذاری منقضی شده است.".to_string(),
                ));
            }

            let mut share_data: ShareForm = serde_json::from_str(&json_string).map_err(|e| {
                error!(target: "system", "JSON deserialization failed: {}", e);
                ShareError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "خطا در پردازش اطلاعات".to_string(),
                )
            })?;

            mutator(&mut share_data)?;

            let new_json = serde_json::to_string(&share_data).map_err(|e| {
                error!(target: "system", "JSON serialization failed: {}", e);
                ShareError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "خطا در ذخیره‌سازی اطلاعات".to_string(),
                )
            })?;

            redis
                .set_ex::<_, _, ()>(&redis_key, &new_json, ttl as u64)
                .await
                .map_err(|e| {
                    error!(target: "system", "Redis SET_EX failed: {}", e);
                    ShareError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "خطا در به‌روزرسانی اطلاعات".to_string(),
                    )
                })?;

            Ok(share_data)
        }
        .await;

        let _: Result<(), _> = redis.del(&lock_key).await;

        return result;
    }

    Err(ShareError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "سرور شلوغ است. لطفاً دوباره تلاش کنید.".to_string(),
    ))
}
