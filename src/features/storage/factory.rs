use crate::features::storage::backend::ftp::FtpBackend;
use crate::features::storage::backend::local::LocalBackend;
use crate::features::storage::backend::FileStorageBackend;
use crate::features::storage::config::{FileUploadConfig, StorageProtocol};
use crate::features::storage::error::StorageError;
use std::sync::Arc;

pub fn build_backend(config: &FileUploadConfig) -> Result<Arc<dyn FileStorageBackend>, StorageError> {
    match config.storage.protocol {
        StorageProtocol::Local => {
            let backend = LocalBackend::new(config.storage.local_root.clone());
            Ok(Arc::new(backend))
        }
        StorageProtocol::Ftp => {
            let backend = FtpBackend::new(config.storage.clone());
            Ok(Arc::new(backend))
        }
    }
}
