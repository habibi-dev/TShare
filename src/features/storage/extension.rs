use crate::features::storage::error::StorageError;
use mime_guess::mime;
use std::collections::HashSet;

fn infer_mime_from_bytes(content: &[u8]) -> mime::Mime {
    if content.starts_with(b"%PDF") {
        return "application/pdf".parse().unwrap();
    }
    if content.len() >= 4 && &content[0..4] == b"\x89PNG" {
        return "image/png".parse().unwrap();
    }
    if content.len() >= 3 && &content[0..3] == b"\xFF\xD8\xFF" {
        return "image/jpeg".parse().unwrap();
    }
    if content.len() >= 6 && (content.starts_with(b"GIF87a") || content.starts_with(b"GIF89a")) {
        return "image/gif".parse().unwrap();
    }
    if content.len() >= 4 && &content[0..4] == b"RIFF" {
        return "image/webp".parse().unwrap();
    }
    if content.starts_with(b"PK\x03\x04") {
        return "application/zip".parse().unwrap();
    }
    "application/octet-stream".parse().unwrap()
}

const MAX_EXTENSION_LEN: usize = 10;

static DENYLIST: &[&str] = &[
    "exe", "bat", "cmd", "com", "msi", "scr", "ps1", "vbs", "js", "jar", "sh", "bash", "php",
    "asp", "aspx", "jsp", "cgi", "dll", "sys", "hta", "reg", "inf", "lnk", "dmg", "app", "deb",
    "rpm", "apk", "html", "htm", "svg", "wasm",
];

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
        validate_mime_consistency(&ext, bytes)?;
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

fn validate_mime_consistency(ext: &str, content: &[u8]) -> Result<(), StorageError> {
    let guessed = infer_mime_from_bytes(content);
    let expected = mime_guess::from_ext(ext).first_or_octet_stream();

    if guessed == expected {
        return Ok(());
    }

    // Allow generic octet-stream when extension is known archive/office
    if guessed.type_() == "application"
        && (guessed.subtype() == "octet-stream" || guessed.subtype() == "zip")
    {
        return Ok(());
    }

    // Reject obvious executable MIME regardless of extension
    let subtype = guessed.subtype().as_str();
    if guessed.type_() == "application"
        && (subtype.contains("executable")
            || subtype == "x-msdownload"
            || subtype == "x-dosexec")
    {
        return Err(StorageError::InvalidExtension(
            "نوع فایل مجاز نیست.".to_string(),
        ));
    }

    // Loose match: same top-level type
    if guessed.type_() == expected.type_() {
        return Ok(());
    }

    Err(StorageError::InvalidExtension(
        "نوع فایل با پسوند آن مطابقت ندارد.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_denylisted_even_if_in_allowed() {
        let mut allowed = HashSet::new();
        allowed.insert("exe".into());
        assert!(validate_upload("malware.exe", &allowed, None).is_err());
    }

    #[test]
    fn accepts_whitelisted_pdf() {
        let mut allowed = HashSet::new();
        allowed.insert("pdf".into());
        assert_eq!(
            validate_upload("doc.pdf", &allowed, None).unwrap(),
            "pdf"
        );
    }
}
