use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    UploadDisabled,
    InvalidExtension(String),
    FileTooLarge,
    Io(String),
    Ftp(String),
    NotFound,
    Misconfigured(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::UploadDisabled => write!(f, "آپلود فایل غیرفعال است"),
            StorageError::InvalidExtension(msg) => write!(f, "{msg}"),
            StorageError::FileTooLarge => write!(f, "حجم فایل بیش از حد مجاز است"),
            StorageError::Io(msg) => write!(f, "خطای ذخیره‌سازی: {msg}"),
            StorageError::Ftp(msg) => write!(f, "خطای FTP: {msg}"),
            StorageError::NotFound => write!(f, "فایل یافت نشد"),
            StorageError::Misconfigured(msg) => write!(f, "پیکربندی storage: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}
