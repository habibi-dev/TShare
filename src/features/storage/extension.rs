use crate::features::storage::error::StorageError;
use std::collections::HashSet;

const MAX_EXTENSION_LEN: usize = 10;

static DENYLIST: &[&str] = &[
    "exe", "bat", "cmd", "com", "msi", "scr", "ps1", "vbs", "js", "jar", "sh", "bash", "php",
    "asp", "aspx", "jsp", "cgi", "dll", "sys", "hta", "reg", "inf", "lnk", "dmg", "app", "deb",
    "rpm", "html", "htm", "svg", "wasm",
];

fn is_zip_container(content: &[u8]) -> bool {
    content.starts_with(b"PK\x03\x04") || content.starts_with(b"PK\x05\x06")
}

fn is_pe_executable(content: &[u8]) -> bool {
    content.starts_with(b"MZ")
}

fn is_elf_executable(content: &[u8]) -> bool {
    content.starts_with(b"\x7FELF")
}

/// Active web/script content smuggled under a benign extension.
fn looks_like_web_script(content: &[u8]) -> bool {
    let sample_len = content.len().min(512);
    if sample_len == 0 {
        return false;
    }
    let sample = String::from_utf8_lossy(&content[..sample_len]).to_ascii_lowercase();
    sample.contains("<!doctype html")
        || sample.contains("<html")
        || sample.starts_with("<?php")
        || sample.contains("<script")
        || sample.contains("javascript:")
}

pub fn parse_allowed_extensions(env: &str) -> HashSet<String> {
    env.split(',')
        .map(|s| s.trim().trim_start_matches('.').to_lowercase())
        .filter(|s| !s.is_empty() && s.len() <= MAX_EXTENSION_LEN)
        .filter(|s| !DENYLIST.contains(&s.as_str()))
        .collect()
}

pub fn validate_upload(
    filename: &str,
    allowed: &HashSet<String>,
    content: Option<&[u8]>,
) -> Result<String, StorageError> {
    if allowed.is_empty() {
        return Err(StorageError::UploadDisabled);
    }

    validate_filename_safe(filename)?;

    let ext = extract_extension(filename)?;
    let ext = normalize_extension(&ext)?;

    if DENYLIST.contains(&ext.as_str()) {
        return Err(StorageError::InvalidExtension(
            "نوع فایل مجاز نیست.".to_string(),
        ));
    }

    if !allowed.contains(&ext) {
        return Err(StorageError::InvalidExtension(
            "نوع فایل مجاز نیست.".to_string(),
        ));
    }

    if let Some(bytes) = content {
        validate_content_safety(&ext, bytes)?;
    }

    Ok(ext)
}

fn validate_filename_safe(filename: &str) -> Result<(), StorageError> {
    let name = filename.trim();
    if name.is_empty() {
        return Err(StorageError::InvalidExtension(
            "نام فایل نامعتبر است.".to_string(),
        ));
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(StorageError::InvalidExtension(
            "نام فایل نامعتبر است.".to_string(),
        ));
    }
    Ok(())
}

fn extract_extension(filename: &str) -> Result<String, StorageError> {
    let base = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    let ext = base
        .rsplit('.')
        .next()
        .filter(|_| base.contains('.'))
        .ok_or_else(|| {
            StorageError::InvalidExtension("فایل باید دارای پسوند باشد.".to_string())
        })?;
    Ok(ext.to_string())
}

fn normalize_extension(ext: &str) -> Result<String, StorageError> {
    let ext = ext.trim().trim_start_matches('.').to_lowercase();
    if ext.is_empty() || ext.len() > MAX_EXTENSION_LEN {
        return Err(StorageError::InvalidExtension(
            "پسوند فایل نامعتبر است.".to_string(),
        ));
    }
    if !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(StorageError::InvalidExtension(
            "پسوند فایل نامعتبر است.".to_string(),
        ));
    }
    Ok(ext)
}

/// Whitelist is enforced by extension; content checks only block obvious abuse.
fn validate_content_safety(ext: &str, content: &[u8]) -> Result<(), StorageError> {
    if content.is_empty() {
        return Err(StorageError::InvalidExtension(
            "فایل خالی مجاز نیست.".to_string(),
        ));
    }

    if ext != "apk" && (is_pe_executable(content) || is_elf_executable(content)) {
        return Err(StorageError::InvalidExtension(
            "نوع فایل مجاز نیست.".to_string(),
        ));
    }

    if ext == "apk" && !is_zip_container(content) {
        return Err(StorageError::InvalidExtension(
            "فایل APK نامعتبر است.".to_string(),
        ));
    }

    if looks_like_web_script(content) {
        return Err(StorageError::InvalidExtension(
            "نوع فایل مجاز نیست.".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed_with(exts: &[&str]) -> HashSet<String> {
        exts.iter().map(|e| e.to_string()).collect()
    }

    #[test]
    fn rejects_denylisted_even_if_in_allowed() {
        let mut allowed = HashSet::new();
        allowed.insert("exe".into());
        assert!(validate_upload("malware.exe", &allowed, None).is_err());
    }

    #[test]
    fn accepts_whitelisted_without_content_check() {
        let allowed = allowed_with(&["png", "pdf", "mp4"]);
        assert_eq!(validate_upload("a.png", &allowed, None).unwrap(), "png");
    }

    #[test]
    fn accepts_png_without_standard_magic() {
        let allowed = allowed_with(&["png"]);
        let content = b"not a real png header but also not malware";
        assert!(validate_upload("photo.png", &allowed, Some(content)).is_ok());
    }

    #[test]
    fn accepts_valid_png_header() {
        let allowed = allowed_with(&["png"]);
        let content = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert!(validate_upload("img.png", &allowed, Some(&content)).is_ok());
    }

    #[test]
    fn rejects_exe_disguised_as_png() {
        let allowed = allowed_with(&["png"]);
        let content = b"MZ\x90\x00fake exe";
        assert!(validate_upload("photo.png", &allowed, Some(content)).is_err());
    }

    #[test]
    fn rejects_html_disguised_as_pdf() {
        let allowed = allowed_with(&["pdf"]);
        let content = b"<!DOCTYPE html><html><script>alert(1)</script>";
        assert!(validate_upload("doc.pdf", &allowed, Some(content)).is_err());
    }

    #[test]
    fn accepts_apk_as_zip() {
        let allowed = allowed_with(&["apk"]);
        let mut content = vec![0x50, 0x4B, 0x03, 0x04];
        content.extend_from_slice(b"android");
        assert!(validate_upload("app.apk", &allowed, Some(&content)).is_ok());
    }

    #[test]
    fn rejects_apk_without_zip_magic() {
        let allowed = allowed_with(&["apk"]);
        let content = b"not an apk";
        assert!(validate_upload("app.apk", &allowed, Some(content)).is_err());
    }
}
