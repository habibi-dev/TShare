use crate::core::response::{json_error, json_success};
use crate::features::setting::service::setting_service::SettingService;
use crate::features::share::utility::generate_unique_key::{generate_unique_key, key_prefix};
use crate::features::share::validation::share_form::ShareForm;
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
use tracing::error;
use validator::Validate;

const DEFAULT_EXPIRY_MINUTES: u64 = 10;
const SECONDS_PER_MINUTE: u64 = 60;

pub struct ShareCreate;

impl ShareCreate {
    pub async fn execute(form: ShareForm) -> Box<Response> {
        // Validate form
        if let Err(e) = form.validate() {
            error!(target: "system", "Form validation failed: {}", e);
            return Box::from(json_error(StatusCode::BAD_REQUEST, e.to_string()));
        }

        // Validate requirements
        if let Err(response) = Self::validate_requirements(&form) {
            return response;
        }

        // Generate unique key
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

        // Prepare and store data
        let data = match Self::prepare_data(&form, &key) {
            Ok(d) => d,
            Err(response) => return response,
        };

        let expiry_seconds = Self::calculate_expiry(&form);

        match Self::store_data(&key, &data, expiry_seconds).await {
            Ok(_) => {
                Self::used_update_concise()
                    .await
                    .expect("Failed to update usage count");

                Box::from(Self::build_response(&key, &data, expiry_seconds))
            }
            Err(e) => {
                error!(target: "system", "Redis storage failed: {}", e);
                Box::from(json_error(StatusCode::INTERNAL_SERVER_ERROR, e))
            }
        }
    }

    fn validate_requirements(form: &ShareForm) -> Result<(), Box<Response>> {
        // Validate password requirement
        if form.require_password.unwrap_or(false) && form.password.is_none() {
            error!(target: "system", "Password required but not provided");
            return Err(Box::from(json_error(
                StatusCode::BAD_REQUEST,
                "پسورد الزامی می باشد.".to_string(),
            )));
        }

        // Validate IP restriction
        if form.restrict_ip.unwrap_or(false) && form.ip.is_none() {
            error!(target: "system", "IP restriction enabled but IP not provided");
            return Err(Box::from(json_error(
                StatusCode::BAD_REQUEST,
                "آدرس آی پی الزامی می باشد.".to_string(),
            )));
        }

        Ok(())
    }

    fn prepare_data(form: &ShareForm, key: &str) -> Result<ShareForm, Box<Response>> {
        let mut data = form.clone();

        // Hash password if required
        if form.require_password.unwrap_or(false)
            && let Some(ref password) = form.password
        {
            data.password = Some(Self::hash_field(password, "password")?);
        }

        // Hash IP if restricted
        if form.restrict_ip.unwrap_or(false)
            && let Some(ref ip) = form.ip
        {
            data.ip = Some(Self::hash_field(ip, "IP")?);
        }

        // Encrypt note
        if let Some(ref note) = form.note {
            data.note = Some(Self::encrypt_note(note, key)?);
        }

        // Generate token
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
