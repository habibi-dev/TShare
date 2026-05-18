use crate::features::storage::error::StorageError;
use async_trait::async_trait;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use tokio::fs::File;

/// How a stored file is exposed for download (stream when possible).
pub enum StoredFile {
    /// Opened on disk — served with a read stream (no full-file RAM buffer).
    Local(File),
    /// Fallback when the backend cannot stream (e.g. FTP retrieve).
    Buffered(Bytes),
}

#[async_trait]
pub trait FileStorageBackend: Send + Sync {
    async fn put_from_path(
        &self,
        remote_name: &str,
        source: &Path,
    ) -> Result<(), StorageError>;

    async fn delete(&self, remote_name: &str) -> Result<(), StorageError>;

    async fn open_for_read(&self, remote_name: &str) -> Result<StoredFile, StorageError>;

    fn local_root(&self) -> Option<&Path> {
        None
    }
}

pub fn upload_temp_dir(local_root: &str) -> PathBuf {
    PathBuf::from(local_root).join(".tmp")
}
