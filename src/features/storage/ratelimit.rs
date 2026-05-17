use crate::features::storage::config::FileRateLimitConfig;
use crate::utility::state::app_state;
use redis::AsyncCommands;
use std::net::IpAddr;

const UPLOAD_COUNT_PREFIX: &str = "rl:upload:count:";
const UPLOAD_BYTES_PREFIX: &str = "rl:upload:bytes:";
const DOWNLOAD_COUNT_PREFIX: &str = "rl:download:count:";
const DOWNLOAD_SHARE_PREFIX: &str = "rl:download:share:";
const SHOW_COUNT_PREFIX: &str = "rl:show:count:";

#[derive(Debug, Clone, Copy)]
pub struct RateLimitExceeded {
    pub retry_after_secs: u64,
}

pub struct FileRateLimiter {
    config: FileRateLimitConfig,
}

impl FileRateLimiter {
    pub fn from_state() -> Self {
        Self {
            config: app_state().file_upload.rate_limit.clone(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn check_upload(
        &self,
        ip: Option<IpAddr>,
        file_size: u64,
    ) -> Result<(), RateLimitExceeded> {
        if !self.config.enabled {
            return Ok(());
        }

        let ip_key = ip_key(ip);

        if self.config.upload_max_per_window > 0 {
            let key = format!("{UPLOAD_COUNT_PREFIX}{ip_key}");
            if !incr_within_limit(
                &key,
                1,
                self.config.upload_max_per_window,
                self.config.upload_window_secs,
            )
            .await
            {
                return Err(RateLimitExceeded {
                    retry_after_secs: self.config.upload_window_secs,
                });
            }
        }

        if self.config.upload_max_bytes_per_window > 0 && file_size > 0 {
            let key = format!("{UPLOAD_BYTES_PREFIX}{ip_key}");
            if !incr_within_limit(
                &key,
                file_size,
                self.config.upload_max_bytes_per_window,
                self.config.upload_bytes_window_secs,
            )
            .await
            {
                if self.config.upload_max_per_window > 0 {
                    let _ = decr_by(&format!("{UPLOAD_COUNT_PREFIX}{ip_key}"), 1).await;
                }
                return Err(RateLimitExceeded {
                    retry_after_secs: self.config.upload_bytes_window_secs,
                });
            }
        }

        Ok(())
    }

    pub async fn check_download(
        &self,
        ip: Option<IpAddr>,
        share_key: &str,
    ) -> Result<(), RateLimitExceeded> {
        if !self.config.enabled {
            return Ok(());
        }

        let ip_key = ip_key(ip);

        if self.config.download_max_per_window > 0 {
            let key = format!("{DOWNLOAD_COUNT_PREFIX}{ip_key}");
            if !incr_within_limit(
                &key,
                1,
                self.config.download_max_per_window,
                self.config.download_window_secs,
            )
            .await
            {
                return Err(RateLimitExceeded {
                    retry_after_secs: self.config.download_window_secs,
                });
            }
        }

        if self.config.download_max_per_share_per_window > 0 {
            let key = format!("{DOWNLOAD_SHARE_PREFIX}{share_key}:{ip_key}");
            if !incr_within_limit(
                &key,
                1,
                self.config.download_max_per_share_per_window,
                self.config.download_share_window_secs,
            )
            .await
            {
                if self.config.download_max_per_window > 0 {
                    let _ =
                        decr_by(&format!("{DOWNLOAD_COUNT_PREFIX}{ip_key}"), 1).await;
                }
                return Err(RateLimitExceeded {
                    retry_after_secs: self.config.download_share_window_secs,
                });
            }
        }

        Ok(())
    }

    pub async fn check_show(&self, ip: Option<IpAddr>) -> Result<(), RateLimitExceeded> {
        if !self.config.enabled {
            return Ok(());
        }

        let ip_key = ip_key(ip);

        if self.config.show_max_per_window > 0 {
            let key = format!("{SHOW_COUNT_PREFIX}{ip_key}");
            if !incr_within_limit(
                &key,
                1,
                self.config.show_max_per_window,
                self.config.show_window_secs,
            )
            .await
            {
                return Err(RateLimitExceeded {
                    retry_after_secs: self.config.show_window_secs,
                });
            }
        }

        Ok(())
    }
}

fn ip_key(ip: Option<IpAddr>) -> String {
    ip.map(|a| a.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

async fn incr_within_limit(key: &str, amount: u64, limit: u64, window_secs: u64) -> bool {
    let state = app_state();
    let mut redis = state.redis.as_ref().clone();

    let new: u64 = match redis.incr(key, amount).await {
        Ok(v) => v,
        Err(_) => return true,
    };

    if new == amount {
        let _: Result<(), _> = redis.expire(key, window_secs as i64).await;
    }

    if new <= limit {
        return true;
    }

    let _ = decr_by(key, amount).await;
    false
}

async fn decr_by(key: &str, amount: u64) -> Result<(), ()> {
    let state = app_state();
    let mut redis = state.redis.as_ref().clone();
    let _: Result<(), _> = redis.decr(key, amount).await;
    Ok(())
}
