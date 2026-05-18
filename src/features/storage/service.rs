use crate::features::storage::backend::r#trait::{StoredFile, upload_temp_dir};
use crate::features::storage::backend::FileStorageBackend;
use crate::features::storage::cleanup::CLEANUP_ZSET_KEY;
use crate::features::storage::config::{FileDownloadMode, FileUploadConfig};
use crate::features::storage::error::StorageError;
use crate::features::storage::extension::validate_upload;
use crate::features::storage::factory::build_backend;
use crate::features::storage::filename::generate_stored_name;
use crate::utility::state::app_state;
use crate::utility::url::url;
use redis::AsyncCommands;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;

const CONTENT_HEADER_BYTES: usize = 512;

/// File staged on disk during multipart parse — not held in RAM.
pub struct UploadedFile {
    pub original_name: String,
    pub temp_path: PathBuf,
    pub size: u64,
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

    pub fn upload_temp_directory(&self) -> PathBuf {
        upload_temp_dir(&self.config.storage.local_root)
    }

    pub async fn upload_file(&self, file: UploadedFile) -> Result<(String, String, u64), StorageError> {
        if !self.config.is_upload_available() {
            let _ = fs::remove_file(&file.temp_path).await;
            return Err(StorageError::UploadDisabled);
        }

        if file.size > self.config.max_size_bytes() {
            let _ = fs::remove_file(&file.temp_path).await;
            return Err(StorageError::FileTooLarge);
        }

        let header = read_file_header(&file.temp_path, CONTENT_HEADER_BYTES).await?;
        let ext = validate_upload(
            &file.original_name,
            &self.config.allowed_extensions,
            Some(&header),
        )?;

        let stored_name = generate_stored_name(&ext);
        if let Err(e) = self
            .backend
            .put_from_path(&stored_name, &file.temp_path)
            .await
        {
            let _ = fs::remove_file(&file.temp_path).await;
            return Err(e);
        }

        Ok((stored_name, file.original_name, file.size))
    }

    pub async fn delete_stored(&self, stored_name: &str) -> Result<(), StorageError> {
        self.backend.delete(stored_name).await?;
        Self::remove_cleanup_entry(stored_name).await;
        Ok(())
    }

    pub async fn open_for_download(&self, stored_name: &str) -> Result<StoredFile, StorageError> {
        self.backend.open_for_read(stored_name).await
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

    pub fn build_download_url(
        &self,
        stored_name: &str,
        share_key: &str,
        force_proxy: bool,
    ) -> Option<String> {
        let mode = if force_proxy {
            FileDownloadMode::Proxy
        } else {
            self.config.download_mode
        };

        match mode {
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

async fn read_file_header(path: &Path, max_len: usize) -> Result<Vec<u8>, StorageError> {
    let mut file = fs::File::open(path)
        .await
        .map_err(|e| StorageError::Io(e.to_string()))?;
    let mut buf = vec![0u8; max_len];
    let n = file
        .read(&mut buf)
        .await
        .map_err(|e| StorageError::Io(e.to_string()))?;
    buf.truncate(n);
    Ok(buf)
}
