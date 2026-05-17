use crate::core::response::{json_error, json_success};
use crate::features::setting::service::setting_service::SettingService;
use crate::features::share::utility::generate_unique_key::{generate_unique_key, key_prefix};
use crate::features::share::validation::share_form::ShareForm;
use crate::features::storage::error::StorageError;
use crate::features::storage::ratelimit::{FileRateLimiter, RateLimitExceeded};
use crate::features::storage::service::{StorageService, UploadedFile};
use crate::utility::encrypt::encrypt;
use crate::utility::hash::hash;
use crate::utility::state::app_state;
use crate::utility::url::url;
use axum::http::StatusCode;
use axum::response::Response;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{Duration, Utc};
use redis::AsyncCommands;
use std::net::IpAddr;
use tracing::error;
use validator::Validate;

const DEFAULT_EXPIRY_MINUTES: u64 = 10;
const SECONDS_PER_MINUTE: u64 = 60;

pub struct ShareCreate;

impl ShareCreate {
    pub async fn execute(
        form: ShareForm,
        file: Option<UploadedFile>,
        client_ip: Option<IpAddr>,
    ) -> Box<Response> {
        if let Err(e) = form.validate() {
            error!(target: "system", "Form validation failed: {}", e);
            return Box::from(json_error(StatusCode::BAD_REQUEST, e.to_string()));
        }

        if let Err(response) = Self::validate_requirements(&form, file.is_some()) {
            return response;
        }

        let key = match generate_unique_key().await {
            Ok(k) => k,
            Err(e) => {
                error!(target: "system", "Failed to generate unique key: {}", e);
                return Box::from(json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to generate key".to_string(),
                ));
            }
        };

        let mut form = form;
        let uploaded_stored_name = if let Some(upload) = file {
            if let Err(response) = Self::check_upload_rate_limit(client_ip, upload.data.len() as u64)
                .await
            {
                return response;
            }

            match Self::handle_file_upload(&upload).await {
                Ok((stored, original, size)) => {
                    form.file_stored_name = Some(stored.clone());
                    form.file_original_name = Some(original);
                    form.file_size = Some(size);
                    Some(stored)
                }
                Err(response) => return response,
            }
        } else {
            None
        };

        let data = match Self::prepare_data(&form, &key) {
            Ok(d) => d,
            Err(response) => {
                if let Some(stored) = uploaded_stored_name {
                    Self::rollback_file(&stored).await;
                }
                return response;
            }
        };

        let expiry_seconds = Self::calculate_expiry(&form);
        let expires_at = Utc::now().timestamp() + expiry_seconds as i64;

        match Self::store_data(&key, &data, expiry_seconds).await {
            Ok(_) => {
                if let Some(ref stored) = data.file_stored_name {
                    StorageService::schedule_cleanup(stored, expires_at).await;
                }

                Self::used_update_concise()
                    .await
                    .expect("Failed to update usage count");

                Box::from(Self::build_response(&key, &data, expiry_seconds))
            }
            Err(e) => {
                if let Some(stored) = uploaded_stored_name {
                    Self::rollback_file(&stored).await;
                }
                error!(target: "system", "Redis storage failed: {}", e);
                Box::from(json_error(StatusCode::INTERNAL_SERVER_ERROR, e))
            }
        }
    }

    async fn check_upload_rate_limit(
        client_ip: Option<IpAddr>,
        file_size: u64,
    ) -> Result<(), Box<Response>> {
        let limiter = FileRateLimiter::from_state();
        if !limiter.is_enabled() {
            return Ok(());
        }

        limiter
            .check_upload(client_ip, file_size)
            .await
            .map_err(rate_limit_to_response)
    }

    async fn handle_file_upload(
        file: &UploadedFile,
    ) -> Result<(String, String, u64), Box<Response>> {
        let storage = StorageService::from_state().map_err(storage_to_response)?;
        storage.upload(file.clone()).await.map_err(storage_to_response)
    }

    async fn rollback_file(stored_name: &str) {
        if let Ok(storage) = StorageService::from_state() {
            let _ = storage.delete_stored(stored_name).await;
        }
    }

    fn validate_requirements(form: &ShareForm, has_file_upload: bool) -> Result<(), Box<Response>> {
        if form.require_password.unwrap_or(false) && form.password.is_none() {
            error!(target: "system", "Password required but not provided");
            return Err(Box::from(json_error(
                StatusCode::BAD_REQUEST,
                "پسورد الزامی می باشد.".to_string(),
            )));
        }

        if form.restrict_ip.unwrap_or(false) && form.ip.is_none() {
            error!(target: "system", "IP restriction enabled but IP not provided");
            return Err(Box::from(json_error(
                StatusCode::BAD_REQUEST,
                "آدرس آی پی الزامی می باشد.".to_string(),
            )));
        }

        let has_note = form
            .note
            .as_ref()
            .map(|n| !n.trim().is_empty())
            .unwrap_or(false);

        if !has_note && !has_file_upload {
            return Err(Box::from(json_error(
                StatusCode::BAD_REQUEST,
                "متن یا فایل الزامی است.".to_string(),
            )));
        }

        if has_file_upload && !app_state().file_upload.is_upload_available() {
            return Err(Box::from(json_error(
                StatusCode::BAD_REQUEST,
                "آپلود فایل غیرفعال است.".to_string(),
            )));
        }

        Ok(())
    }

    fn prepare_data(form: &ShareForm, key: &str) -> Result<ShareForm, Box<Response>> {
        let mut data = form.clone();

        if form.require_password.unwrap_or(false)
            && let Some(ref password) = form.password
        {
            data.password = Some(Self::hash_field(password, "password")?);
        }

        if form.restrict_ip.unwrap_or(false)
            && let Some(ref ip) = form.ip
        {
            data.ip = Some(Self::hash_field(ip, "IP")?);
        }

        if let Some(ref note) = form.note {
            if !note.trim().is_empty() {
                data.note = Some(Self::encrypt_note(note, key)?);
            } else {
                data.note = None;
            }
        }

        data.token = Some(Self::hash_field(key, "token")?);

        Ok(data)
    }

    fn hash_field(value: &str, field_name: &str) -> Result<String, Box<Response>> {
        hash(value).map_err(|e| {
            error!(target: "system", "{} hashing failed: {}", field_name, e);
            Box::from(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to hash {}", field_name),
            ))
        })
    }

    fn encrypt_note(note: &str, key: &str) -> Result<String, Box<Response>> {
        encrypt(note, key).map_err(|e| {
            error!(target: "system", "Note encryption failed: {}", e);
            Box::from(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encrypt note".to_string(),
            ))
        })
    }

    fn calculate_expiry(form: &ShareForm) -> u64 {
        match &form.expiry {
            Some(exp) => exp.as_seconds(),
            None => DEFAULT_EXPIRY_MINUTES * SECONDS_PER_MINUTE,
        }
    }

    async fn store_data(key: &str, data: &ShareForm, expiry: u64) -> Result<(), String> {
        let state = app_state();
        let mut redis = state.redis.as_ref().clone();
        let redis_key = key_prefix(&key.to_string());

        let json_string = serde_json::to_string(&data).map_err(|e| {
            error!(target: "system", "JSON serialization failed: {}", e);
            format!("Serialization failed: {}", e)
        })?;

        redis
            .set_ex::<_, _, ()>(redis_key, &json_string, expiry)
            .await
            .map_err(|e| {
                error!(target: "system", "Redis SET_EX failed: {}", e);
                format!("Redis error: {}", e)
            })?;

        Ok(())
    }

    fn build_response(key: &str, data: &ShareForm, expiry: u64) -> Response {
        let token = data
            .token
            .as_ref()
            .map(|t| BASE64_STANDARD.encode(t))
            .unwrap_or_default();
        let json_data = serde_json::json!({
            "id": key,
            "url": url(&format!("/c/{}", &key)),
            "token": token,
            "expiry": Utc::now() + Duration::seconds(expiry as i64)
        });

        json_success(json_data)
    }

    async fn used_update_concise() -> Result<i64, Box<dyn std::error::Error>> {
        let new_value = SettingService::get_by_key("used")
            .await?
            .and_then(|s| s.meta_value)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0)
            + 1;

        SettingService::upsert("used".to_string(), Some(new_value.to_string())).await?;

        Ok(new_value)
    }
}

fn rate_limit_to_response(err: RateLimitExceeded) -> Box<Response> {
    Box::from(json_error(
        StatusCode::TOO_MANY_REQUESTS,
        format!(
            "تعداد درخواست‌های آپلود شما بیش از حد مجاز است. لطفاً {} ثانیه دیگر تلاش کنید.",
            err.retry_after_secs
        ),
    ))
}

fn storage_to_response(err: StorageError) -> Box<Response> {
    let status = match &err {
        StorageError::FileTooLarge | StorageError::InvalidExtension(_) => StatusCode::BAD_REQUEST,
        StorageError::UploadDisabled => StatusCode::BAD_REQUEST,
        StorageError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    Box::from(json_error(status, err.to_string()))
}
