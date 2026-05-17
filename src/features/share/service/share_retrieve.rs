use crate::features::share::service::share_atomic::update_share_atomic;
use crate::features::share::utility::generate_unique_key::key_prefix;
use crate::features::share::validation::share_form::ShareForm;
use crate::features::share::validation::share_show::ShowRequest;
use crate::utility::encrypt::decrypt;
use crate::utility::hash::verify;
use crate::utility::state::app_state;
use axum::http::StatusCode;
use redis::AsyncCommands;
use tracing::error;

pub struct ShareRetrieve;

#[derive(Debug, Clone)]
pub struct ShareError {
    pub code: String,
    pub title: String,
    pub message: String,
}

impl ShareError {
    pub fn new(status: StatusCode, message: String) -> Self {
        let (code, title) = match status {
            StatusCode::NOT_FOUND => ("404".to_string(), "یافت نشد".to_string()),
            StatusCode::UNAUTHORIZED => ("401".to_string(), "عدم دسترسی".to_string()),
            StatusCode::FORBIDDEN => ("403".to_string(), "ممنوع".to_string()),
            StatusCode::GONE => ("410".to_string(), "منقضی شده".to_string()),
            StatusCode::INTERNAL_SERVER_ERROR => ("500".to_string(), "خطای سرور".to_string()),
            _ => (status.as_str().to_string(), "خطا".to_string()),
        };

        Self {
            code,
            title,
            message,
        }
    }
}

impl ShareRetrieve {
    pub async fn authorize_download(request: &ShowRequest) -> Result<ShareForm, ShareError> {
        let share_data = Self::get_data(&request.key).await?;
        Self::validate_access(&share_data, request)?;
        Self::check_download_limits(&share_data)?;
        Ok(share_data)
    }

    /// Atomically validates access and marks a one-time file share as downloaded.
    pub async fn consume_download(request: &ShowRequest) -> Result<ShareForm, ShareError> {
        let key = request.key.clone();
        let password = request.password.clone();
        let ip = request.ip.clone();

        update_share_atomic(&key, |share| {
            let req = ShowRequest {
                key: key.clone(),
                password: password.clone(),
                ip: ip.clone(),
            };
            Self::validate_access(share, &req)?;
            Self::check_download_limits(share)?;
            share.downloaded = Some(true);
            Ok(())
        })
        .await
    }

    pub async fn access_file_download(request: &ShowRequest) -> Result<ShareForm, ShareError> {
        let peek = Self::get_data(&request.key).await?;
        if peek.file_stored_name.is_none() {
            return Err(ShareError::new(
                StatusCode::NOT_FOUND,
                "فایلی برای این اشتراک وجود ندارد.".to_string(),
            ));
        }
        if peek.one_time_use.unwrap_or(false) {
            Self::consume_download(request).await
        } else {
            Self::authorize_download(request).await
        }
    }

    pub async fn execute(request: ShowRequest) -> Result<ShareForm, ShareError> {
        let key = request.key.clone();
        let password = request.password.clone();
        let ip = request.ip.clone();

        let mut share_data = update_share_atomic(&key, |share| {
            let req = ShowRequest {
                key: key.clone(),
                password: password.clone(),
                ip: ip.clone(),
            };
            Self::validate_access(share, &req)?;
            Self::check_show_limits(share)?;
            share.viewed = Some(share.viewed.unwrap_or(0) + 1);
            Ok(())
        })
        .await?;

        Self::decrypt_note(&mut share_data, &key)?;
        Ok(share_data)
    }

    pub(crate) async fn get_data(key: &str) -> Result<ShareForm, ShareError> {
        let state = app_state();
        let mut redis = state.redis.as_ref().clone();
        let redis_key = key_prefix(&key.to_string());

        let json_string: String = redis.get(&redis_key).await.map_err(|e| {
            error!(target: "system", "Redis GET failed for key {}: {}", key, e);
            ShareError::new(
                StatusCode::NOT_FOUND,
                "اشتراک گذاری یافت نشد یا منقضی شده است.".to_string(),
            )
        })?;

        if json_string.is_empty() {
            error!(target: "system", "Share not found: {}", key);
            return Err(ShareError::new(
                StatusCode::NOT_FOUND,
                "اشتراک گذاری یافت نشد یا منقضی شده است.".to_string(),
            ));
        }

        serde_json::from_str(&json_string).map_err(|e| {
            error!(target: "system", "JSON deserialization failed: {}", e);
            ShareError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "خطا در پردازش اطلاعات".to_string(),
            )
        })
    }

    fn validate_access(share_data: &ShareForm, request: &ShowRequest) -> Result<(), ShareError> {
        Self::verify_password(share_data, &request.password)?;
        Self::verify_ip(share_data, &request.ip)?;
        Ok(())
    }

    fn verify_password(
        share_data: &ShareForm,
        provided_password: &Option<String>,
    ) -> Result<(), ShareError> {
        if !share_data.require_password.unwrap_or(false) {
            return Ok(());
        }

        let stored_hash = share_data.password.as_ref().ok_or_else(|| {
            error!(target: "system", "Password required but hash not found");
            ShareError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "خطا در پیکربندی سیستم".to_string(),
            )
        })?;

        let provided = provided_password.as_ref().ok_or_else(|| {
            ShareError::new(
                StatusCode::UNAUTHORIZED,
                "برای مشاهده این متن وارد کردن پسورد الزامی می باشد.".to_string(),
            )
        })?;

        match verify(provided, stored_hash) {
            Ok(true) => Ok(()),
            Ok(false) => {
                error!(target: "system", "Invalid password provided");
                Err(ShareError::new(
                    StatusCode::UNAUTHORIZED,
                    "پسورد نادرست است.".to_string(),
                ))
            }
            Err(e) => {
                error!(target: "system", "Password verification failed: {}", e);
                Err(ShareError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "خطا در بررسی رمز عبور".to_string(),
                ))
            }
        }
    }

    fn verify_ip(share_data: &ShareForm, request_ip: &str) -> Result<(), ShareError> {
        if !share_data.restrict_ip.unwrap_or(false) {
            return Ok(());
        }

        let stored_hash = share_data.ip.as_ref().ok_or_else(|| {
            error!(target: "system", "IP restriction enabled but hash not found");
            ShareError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "خطا در پیکربندی سیستم".to_string(),
            )
        })?;

        match verify(request_ip, stored_hash) {
            Ok(true) => Ok(()),
            Ok(false) => {
                error!(target: "system", "IP mismatch for request from: {}", request_ip);
                Err(ShareError::new(
                    StatusCode::FORBIDDEN,
                    "دسترسی از این آی پی مجاز نیست.".to_string(),
                ))
            }
            Err(e) => {
                error!(target: "system", "IP verification failed: {}", e);
                Err(ShareError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "خطا در بررسی آی پی".to_string(),
                ))
            }
        }
    }

    fn has_attached_file(share_data: &ShareForm) -> bool {
        share_data.file_stored_name.is_some()
    }

    fn one_time_consumed_error() -> ShareError {
        ShareError::new(
            StatusCode::GONE,
            "این اشتراک گذاری یکبار مصرف بوده و قبلاً استفاده شده است.".to_string(),
        )
    }

    fn check_show_limits(share_data: &ShareForm) -> Result<(), ShareError> {
        let current_views = share_data.viewed.unwrap_or(0);
        let one_time = share_data.one_time_use.unwrap_or(false);

        if one_time {
            if Self::has_attached_file(share_data) {
                if share_data.downloaded.unwrap_or(false) {
                    error!(target: "system", "One-time file share already downloaded");
                    return Err(Self::one_time_consumed_error());
                }
            } else if current_views >= 1 {
                error!(target: "system", "One-time text share already viewed");
                return Err(Self::one_time_consumed_error());
            }
        }

        if let Some(ref max_views) = share_data.max_views {
            let limit = max_views.as_int();

            if limit != -1 && current_views >= limit as u64 {
                error!(target: "system", "View limit reached: {}/{}", current_views, limit);
                return Err(ShareError::new(
                    StatusCode::GONE,
                    "حداکثر تعداد مشاهده به پایان رسیده است.".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn check_download_limits(share_data: &ShareForm) -> Result<(), ShareError> {
        if share_data.one_time_use.unwrap_or(false) && share_data.downloaded.unwrap_or(false) {
            error!(target: "system", "One-time file share already downloaded");
            return Err(Self::one_time_consumed_error());
        }

        Ok(())
    }

    fn decrypt_note(share_data: &mut ShareForm, key: &str) -> Result<(), ShareError> {
        let Some(encrypted) = share_data.note.as_ref() else {
            return Ok(());
        };

        if encrypted.is_empty() {
            share_data.note = None;
            return Ok(());
        }

        let decrypted_note = decrypt(encrypted, key).map_err(|e| {
            error!(target: "system", "Decryption failed: {}", e);
            ShareError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "خطا در رمزگشایی محتوا".to_string(),
            )
        })?;

        share_data.note = Some(decrypted_note);

        Ok(())
    }
}
