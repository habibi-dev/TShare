use crate::features::storage::error::StorageError;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait FileStorageBackend: Send + Sync {
    async fn put(&self, remote_name: &str, data: Bytes) -> Result<(), StorageError>;
    async fn delete(&self, remote_name: &str) -> Result<(), StorageError>;
    async fn get(&self, remote_name: &str) -> Result<Bytes, StorageError>;
}
