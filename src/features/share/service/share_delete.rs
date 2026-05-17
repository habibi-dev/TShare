use crate::core::response::{json_error, json_success};
use crate::features::share::service::share_retrieve::ShareRetrieve;
use crate::features::share::service::share_update::ShareUpdate;
use crate::features::share::utility::generate_unique_key::key_prefix;
use crate::features::share::validation::share_delete::DeleteRequest;
use crate::features::storage::service::StorageService;
use crate::utility::state::app_state;
use axum::http::StatusCode;
use axum::response::Response;
use redis::AsyncCommands;
use tracing::error;
use validator::Validate;

pub struct ShareDelete;

impl ShareDelete {
    pub async fn execute(request: DeleteRequest) -> Box<Response> {
        // Validate request
        if let Err(e) = request.validate() {
            error!(target: "system", "Delete request validation failed: {}", e);
            return Box::from(json_error(StatusCode::BAD_REQUEST, e.to_string()));
        }

        // Get share data
        let share_data = match ShareRetrieve::get_data(&request.key).await {
            Ok(data) => data,
            Err(response) => {
                return Box::from(json_error(response.code.parse().unwrap(), response.title));
            }
        };

        // Verify token
        if let Err(response) = ShareUpdate::verify_token(&request.token, &share_data) {
            return response;
        }

        if let Some(ref stored) = share_data.file_stored_name {
            if let Ok(storage) = StorageService::from_state() {
                let _ = storage.delete_stored(stored).await;
            }
        }

        // Delete from Redis
        match Self::delete_data(&request.key).await {
            Ok(_) => Box::from(json_success(serde_json::json!({
                "message": "اشتراک گذاری با موفقیت حذف شد.",
                "key": request.key
            }))),
            Err(response) => Box::from(response),
        }
    }

    async fn delete_data(key: &str) -> Result<(), Response> {
        let state = app_state();
        let mut redis = state.redis.as_ref().clone();
        let redis_key = key_prefix(&key.to_string());

        let deleted: u32 = redis.del(&redis_key).await.map_err(|e| {
            error!(target: "system", "Redis DELETE failed: {}", e);
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete data".to_string(),
            )
        })?;

        if deleted == 0 {
            error!(target: "system", "No data was deleted for key: {}", key);
            return Err(json_error(
                StatusCode::NOT_FOUND,
                "اشتراک گذاری یافت نشد.".to_string(),
            ));
        }

        Ok(())
    }
}
