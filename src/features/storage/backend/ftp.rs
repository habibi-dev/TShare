use crate::features::storage::backend::FileStorageBackend;
use crate::features::storage::config::StorageBackendSettings;
use crate::features::storage::error::StorageError;
use async_trait::async_trait;
use bytes::Bytes;
use std::io::Cursor;
use std::sync::Arc;
use suppaftp::FtpStream;

#[derive(Clone)]
pub struct FtpBackend {
    settings: Arc<StorageBackendSettings>,
}

impl FtpBackend {
    pub fn new(settings: StorageBackendSettings) -> Self {
        Self {
            settings: Arc::new(settings),
        }
    }

    fn remote_path(&self, remote_name: &str) -> String {
        let base = self.settings.base_path.trim();
        let base = base.trim_end_matches('/');
        if base.is_empty() || base == "/" {
            remote_name.to_string()
        } else {
            format!("{}/{}", base.trim_start_matches('/'), remote_name)
        }
    }

    fn connect(&self) -> Result<FtpStream, StorageError> {
        if self.settings.host.is_empty() {
            return Err(StorageError::Misconfigured(
                "FILE_STORAGE_HOST is required for FTP".to_string(),
            ));
        }

        let addr = format!("{}:{}", self.settings.host, self.settings.port);
        let mut ftp = FtpStream::connect(&addr).map_err(|e| StorageError::Ftp(e.to_string()))?;

        if self.settings.use_tls {
            return Err(StorageError::Misconfigured(
                "FTP explicit TLS is not supported yet; use FILE_STORAGE_USE_TLS=false or local storage"
                    .to_string(),
            ));
        }

        ftp.login(
            &self.settings.username,
            &self.settings.password,
        )
        .map_err(|e| StorageError::Ftp(e.to_string()))?;

        Ok(ftp)
    }
}

#[async_trait]
impl FileStorageBackend for FtpBackend {
    async fn put(&self, remote_name: &str, data: Bytes) -> Result<(), StorageError> {
        let path = self.remote_path(remote_name);
        let settings = self.settings.clone();
        let bytes = data.to_vec();

        tokio::task::spawn_blocking(move || {
            let backend = FtpBackend::new(settings.as_ref().clone());
            let mut ftp = backend.connect()?;
            let mut reader = Cursor::new(bytes);
            ftp.put_file(&path, &mut reader)
                .map_err(|e| StorageError::Ftp(e.to_string()))?;
            let _ = ftp.quit();
            Ok::<(), StorageError>(())
        })
        .await
        .map_err(|e| StorageError::Io(e.to_string()))??;

        Ok(())
    }

    async fn delete(&self, remote_name: &str) -> Result<(), StorageError> {
        let path = self.remote_path(remote_name);
        let settings = self.settings.clone();

        tokio::task::spawn_blocking(move || {
            let backend = FtpBackend::new(settings.as_ref().clone());
            let mut ftp = backend.connect()?;
            ftp.rm(&path)
                .map_err(|e| StorageError::Ftp(e.to_string()))?;
            let _ = ftp.quit();
            Ok::<(), StorageError>(())
        })
        .await
        .map_err(|e| StorageError::Io(e.to_string()))??;

        Ok(())
    }

    async fn get(&self, remote_name: &str) -> Result<Bytes, StorageError> {
        let path = self.remote_path(remote_name);
        let settings = self.settings.clone();

        let data = tokio::task::spawn_blocking(move || {
            let backend = FtpBackend::new(settings.as_ref().clone());
            let mut ftp = backend.connect()?;
            let data = ftp
                .retr(&path, |reader| {
                    let mut writer = Vec::new();
                    std::io::copy(reader, &mut writer).map_err(|e| {
                        suppaftp::FtpError::ConnectionError(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ))
                    })?;
                    Ok(writer)
                })
                .map_err(|e| StorageError::Ftp(e.to_string()))?;
            let _ = ftp.quit();
            Ok::<Vec<u8>, StorageError>(data)
        })
        .await
        .map_err(|e| StorageError::Io(e.to_string()))??;

        Ok(Bytes::from(data))
    }
}
