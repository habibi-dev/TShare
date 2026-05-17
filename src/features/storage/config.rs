use crate::features::storage::extension::parse_allowed_extensions;
use std::collections::HashSet;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDownloadMode {
    Direct,
    Proxy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageProtocol {
    Ftp,
    Local,
}

#[derive(Debug, Clone)]
pub struct StorageBackendSettings {
    pub protocol: StorageProtocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub base_path: String,
    pub use_tls: bool,
    pub local_root: String,
}

#[derive(Debug, Clone)]
pub struct FileUploadConfig {
    pub upload_enabled: bool,
    pub max_size_mb: u64,
    pub public_base_url: String,
    pub download_mode: FileDownloadMode,
    pub allowed_extensions: HashSet<String>,
    pub storage: StorageBackendSettings,
    pub cleanup_interval_secs: u64,
}

impl FileUploadConfig {
    pub fn from_env() -> Self {
        let upload_enabled = env::var("FILE_UPLOAD_ENABLED")
            .unwrap_or_else(|_| "false".into())
            .parse()
            .unwrap_or(false);

        let max_size_mb = env::var("FILE_MAX_SIZE_MB")
            .unwrap_or_else(|_| "10".into())
            .parse()
            .unwrap_or(10);

        let public_base_url = env::var("FILE_PUBLIC_BASE_URL").unwrap_or_default();

        let download_mode = match env::var("FILE_DOWNLOAD_MODE")
            .unwrap_or_else(|_| "direct".into())
            .to_lowercase()
            .as_str()
        {
            "proxy" => FileDownloadMode::Proxy,
            _ => FileDownloadMode::Direct,
        };

        let allowed_extensions =
            parse_allowed_extensions(&env::var("FILE_ALLOWED_EXTENSIONS").unwrap_or_default());

        let protocol = match env::var("FILE_STORAGE_PROTOCOL")
            .unwrap_or_else(|_| "local".into())
            .to_lowercase()
            .as_str()
        {
            "ftp" => StorageProtocol::Ftp,
            _ => StorageProtocol::Local,
        };

        let local_root = env::var("FILE_STORAGE_LOCAL_ROOT").unwrap_or_else(|_| "uploads".into());

        let storage = StorageBackendSettings {
            protocol,
            host: env::var("FILE_STORAGE_HOST").unwrap_or_default(),
            port: env::var("FILE_STORAGE_PORT")
                .unwrap_or_else(|_| "21".into())
                .parse()
                .unwrap_or(21),
            username: env::var("FILE_STORAGE_USERNAME").unwrap_or_default(),
            password: env::var("FILE_STORAGE_PASSWORD").unwrap_or_default(),
            base_path: env::var("FILE_STORAGE_BASE_PATH").unwrap_or_else(|_| "/".into()),
            use_tls: env::var("FILE_STORAGE_USE_TLS")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            local_root,
        };

        let cleanup_interval_secs = env::var("FILE_CLEANUP_INTERVAL_SECS")
            .unwrap_or_else(|_| "60".into())
            .parse()
            .unwrap_or(60);

        Self {
            upload_enabled,
            max_size_mb,
            public_base_url,
            download_mode,
            allowed_extensions,
            storage,
            cleanup_interval_secs,
        }
    }

    pub fn is_upload_available(&self) -> bool {
        self.upload_enabled && !self.allowed_extensions.is_empty()
    }

    pub fn max_size_bytes(&self) -> u64 {
        self.max_size_mb.saturating_mul(1024 * 1024)
    }

    pub fn accept_attr(&self) -> String {
        self.allowed_extensions
            .iter()
            .map(|ext| format!(".{ext}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn extensions_display(&self) -> String {
        let mut list: Vec<&str> = self.allowed_extensions.iter().map(String::as_str).collect();
        list.sort();
        list.join(", ")
    }
}
