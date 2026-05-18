use crate::features::storage::backend::r#trait::StoredFile;
use crate::features::storage::backend::FileStorageBackend;
use crate::features::storage::error::StorageError;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Clone)]
pub struct LocalBackend {
    root: PathBuf,
}

impl LocalBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
        }
    }

    fn full_path(&self, remote_name: &str) -> Result<PathBuf, StorageError> {
        if remote_name.contains("..") || remote_name.contains('/') || remote_name.contains('\\') {
            return Err(StorageError::InvalidExtension(
                "نام فایل نامعتبر است.".to_string(),
            ));
        }
        Ok(self.root.join(remote_name))
    }
}

#[async_trait]
impl FileStorageBackend for LocalBackend {
    async fn put_from_path(
        &self,
        remote_name: &str,
        source: &Path,
    ) -> Result<(), StorageError> {
        let path = self.full_path(remote_name)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::Io(e.to_string()))?;
        }

        match fs::rename(source, &path).await {
            Ok(()) => Ok(()),
            Err(_) => {
                fs::copy(source, &path)
                    .await
                    .map_err(|e| StorageError::Io(e.to_string()))?;
                let _ = fs::remove_file(source).await;
                Ok(())
            }
        }
    }

    async fn delete(&self, remote_name: &str) -> Result<(), StorageError> {
        let path = self.full_path(remote_name)?;
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e.to_string())),
        }
    }

    async fn open_for_read(&self, remote_name: &str) -> Result<StoredFile, StorageError> {
        let path = self.full_path(remote_name)?;
        let file = fs::File::open(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound
            } else {
                StorageError::Io(e.to_string())
            }
        })?;
        Ok(StoredFile::Local(file))
    }

    fn local_root(&self) -> Option<&Path> {
        Some(&self.root)
    }
}
