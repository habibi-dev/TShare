use crate::core::response::json_error;
use crate::features::share::validation::share_form::ShareForm;
use axum::http::StatusCode;
use axum::response::Response;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::error;

pub struct ShareUpdate;

impl ShareUpdate {
    /*    pub async fn execute(request: UpdateRequest) -> Box<Response> {
            // Validate request
            if let Err(e) = request.validate() {
                error!(target: "system", "Update request validation failed: {}", e);
                return Box::from(json_error(StatusCode::BAD_REQUEST, e.to_string()));
            }

            // Get existing share data
            let mut share_data = match ShareRetrieve::get_data(&request.key).await {
                Ok(data) => data,
                Err(response) => return Box::from(response),
            };

            // Verify token
            if let Err(response) = Self::verify_token(&request.token, &share_data) {
                return response;
            }

            // Update fields
            if let Err(response) = Self::update_fields(&mut share_data, &request).await {
                return Box::from(response);
            }

            // Save updated data
            match Self::save_data(&request.key, &share_data).await {
                Ok(_) => Box::from(json_success(serde_json::json!({
                    "message": "اشتراک گذاری با موفقیت به‌روزرسانی شد.",
                    "key": request.key
                }))),
                Err(response) => Box::from(response),
            }
        }
    */
    pub fn verify_token(provided_token: &str, share_data: &ShareForm) -> Result<(), Box<Response>> {
        // Decode token from base64
        let decoded_token = BASE64_STANDARD.decode(provided_token).map_err(|e| {
            error!(target: "system", "Token base64 decode failed: {}", e);
            json_error(StatusCode::BAD_REQUEST, "توکن نامعتبر است.".to_string())
        })?;

        let decoded_token_str = String::from_utf8(decoded_token).map_err(|e| {
            error!(target: "system", "Token UTF-8 conversion failed: {}", e);
            json_error(StatusCode::BAD_REQUEST, "توکن نامعتبر است.".to_string())
        })?;

        // Get stored token hash
        let stored_hash = share_data.token.as_ref().ok_or_else(|| {
            error!(target: "system", "No token found in share data");
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
            )
        })?;

        // Verify token matches stored hash
        if &decoded_token_str != stored_hash {
            error!(target: "system", "Token verification failed");
            return Err(Box::from(json_error(
                StatusCode::UNAUTHORIZED,
                "توکن نامعتبر است.".to_string(),
            )));
        }

        Ok(())
    }

    /*    async fn update_fields(
        share_data: &mut ShareForm,
        request: &UpdateRequest,
    ) -> Result<(), Response> {
        // Update note if provided
        if let Some(ref new_note) = request.note {
            // Decrypt old note first to get the key
            let _old_encrypted = share_data.note.as_ref().ok_or_else(|| {
                error!(target: "system", "No existing note found");
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error".to_string(),
                )
            })?;

            // Encrypt new note with the same key (request.key is the encryption key)
            let encrypted = encrypt(new_note, &request.key).map_err(|e| {
                error!(target: "system", "Note encryption failed: {}", e);
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to encrypt note".to_string(),
                )
            })?;

            share_data.note = Some(encrypted);
        }

        // Update max_views if provided
        if let Some(ref max_views) = request.max_views {
            share_data.max_views = Some(max_views.clone());
        }

        // Update one_time_use if provided
        if let Some(one_time_use) = request.one_time_use {
            share_data.one_time_use = Some(one_time_use);
        }

        // Update password if provided
        if let Some(require_password) = request.require_password {
            share_data.require_password = Some(require_password);

            if require_password {
                if let Some(ref new_password) = request.password {
                    let hashed = hash(new_password).map_err(|e| {
                        error!(target: "system", "Password hashing failed: {}", e);
                        json_error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to hash password".to_string(),
                        )
                    })?;
                    share_data.password = Some(hashed);
                } else {
                    return Err(json_error(
                        StatusCode::BAD_REQUEST,
                        "پسورد الزامی می باشد.".to_string(),
                    ));
                }
            } else {
                share_data.password = None;
            }
        }

        Ok(())
    }*/

    /*    async fn save_data(key: &str, share_data: &ShareForm) -> Result<(), Response> {
            let state = app_state();
            let mut redis = state.redis.as_ref().clone();
            let redis_key = key_prefix(&key.to_string());

            // Get current TTL to preserve expiry
            let ttl = Self::get_current_ttl_redis(&mut redis, &redis_key).await?;

            if ttl <= 0 {
                error!(target: "system", "Share expired during update");
                return Err(json_error(
                    StatusCode::GONE,
                    "اشتراک گذاری منقضی شده است.".to_string(),
                ));
            }

            // Serialize updated data
            let json_string = serde_json::to_string(&share_data).map_err(|e| {
                error!(target: "system", "JSON serialization failed: {}", e);
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to serialize data".to_string(),
                )
            })?;

            // Update with same TTL
            redis
                .set_ex::<_, _, ()>(&redis_key, &json_string, ttl as u64)
                .await
                .map_err(|e| {
                    error!(target: "system", "Redis update failed: {}", e);
                    json_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to update data".to_string(),
                    )
                })?;

            Ok(())
        }
    */
    pub async fn get_current_ttl_redis(
        redis: &mut ConnectionManager,
        redis_key: &String,
    ) -> Result<i64, Response> {
        let ttl: i64 = redis.ttl(redis_key).await.map_err(|e| {
            error!(target: "system", "Failed to get TTL: {}", e);
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update data".to_string(),
            )
        })?;
        Ok(ttl)
    }
}
