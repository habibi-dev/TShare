use crate::features::storage::backend::FileStorageBackend;
use crate::features::storage::cleanup::CLEANUP_ZSET_KEY;
use crate::features::storage::config::{FileDownloadMode, FileUploadConfig};
use crate::features::storage::error::StorageError;
use crate::features::storage::extension::validate_upload;
use crate::features::storage::factory::build_backend;
use crate::features::storage::filename::generate_stored_name;
use crate::utility::state::app_state;
use crate::utility::url::url;
use bytes::Bytes;
use redis::AsyncCommands;
use std::sync::Arc;

#[derive(Clone)]
pub struct UploadedFile {
    pub original_name: String,
    pub data: Bytes,
}

pub struct StorageService {
    backend: Arc<dyn FileStorageBackend>,
    config: FileUploadConfig,
}

impl StorageService {
    pub fn new(config: FileUploadConfig) -> Result<Self, StorageError> {
        let backend = build_backend(&config)?;
        Ok(Self { backend, config })
    }

    pub fn from_state() -> Result<Self, StorageError> {
        let state = app_state();
        Self::new(state.file_upload.clone())
    }

    pub fn config(&self) -> &FileUploadConfig {
        &self.config
    }

    pub async fn upload(&self, file: UploadedFile) -> Result<(String, String, u64), StorageError> {
        if !self.config.is_upload_available() {
            return Err(StorageError::UploadDisabled);
        }

        if file.data.len() as u64 > self.config.max_size_bytes() {
            return Err(StorageError::FileTooLarge);
        }

        let ext = validate_upload(
            &file.original_name,
            &self.config.allowed_extensions,
            Some(&file.data),
        )?;

        let stored_name = generate_stored_name(&ext);
        self.backend.put(&stored_name, file.data.clone()).await?;

        Ok((stored_name, file.original_name, file.data.len() as u64))
    }

    pub async fn delete_stored(&self, stored_name: &str) -> Result<(), StorageError> {
        self.backend.delete(stored_name).await?;
        Self::remove_cleanup_entry(stored_name).await;
        Ok(())
    }

    pub async fn get_stored(&self, stored_name: &str) -> Result<Bytes, StorageError> {
        self.backend.get(stored_name).await
    }

    pub async fn schedule_cleanup(stored_name: &str, expires_at_unix: i64) {
        let state = app_state();
        let mut redis = state.redis.as_ref().clone();
        let _: Result<(), _> = redis
            .zadd(CLEANUP_ZSET_KEY, stored_name, expires_at_unix as f64)
            .await;
    }

    pub async fn remove_cleanup_entry(stored_name: &str) {
        let state = app_state();
        let mut redis = state.redis.as_ref().clone();
        let _: Result<(), _> = redis.zrem(CLEANUP_ZSET_KEY, stored_name).await;
    }

    pub fn build_download_url(&self, stored_name: &str, share_key: &str) -> Option<String> {
        match self.config.download_mode {
            FileDownloadMode::Direct => {
                let base = self.config.public_base_url.trim();
                if base.is_empty() {
                    return None;
                }
                let base = if base.ends_with('/') {
                    base.to_string()
                } else {
                    format!("{base}/")
                };
                Some(format!("{base}{stored_name}"))
            }
            FileDownloadMode::Proxy => Some(url(&format!("/c/{share_key}/file"))),
        }
    }

    pub fn display_filename(original: &str) -> String {
        original
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(original)
            .to_string()
    }
}
