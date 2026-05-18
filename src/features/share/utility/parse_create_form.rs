use crate::core::response::json_error;
use crate::features::share::validation::share_form::{Expiry, MaxViews, ShareForm};
use crate::features::storage::service::UploadedFile;
use crate::utility::state::app_state;
use axum::extract::multipart::Field;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::Response;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub struct CreateShareInput {
    pub form: ShareForm,
    pub file: Option<UploadedFile>,
}

const MAX_NOTE_LEN: usize = 10_000;
const MAX_PASSWORD_LEN: usize = 24;
const MAX_IP_FIELD_LEN: usize = 45;
const MAX_TEXT_FIELD_LEN: usize = 64;

fn parse_bool(value: Option<&str>) -> Option<bool> {
    value.map(|v| {
        let v = v.trim().to_lowercase();
        v == "true" || v == "on" || v == "1"
    })
}

fn parse_expiry(value: Option<&str>) -> Option<Expiry> {
    match value?.trim() {
        "5" => Some(Expiry::N5),
        "10" => Some(Expiry::N10),
        "30" => Some(Expiry::N30),
        "60" => Some(Expiry::N60),
        "180" => Some(Expiry::N180),
        "360" => Some(Expiry::N360),
        "720" => Some(Expiry::N720),
        "1440" => Some(Expiry::N1440),
        _ => None,
    }
}

fn parse_max_views(value: Option<&str>) -> Option<MaxViews> {
    match value?.trim() {
        "v1" => Some(MaxViews::V1),
        "v5" => Some(MaxViews::V5),
        "v10" => Some(MaxViews::V10),
        "unlimited" => Some(MaxViews::UNLIMITED),
        _ => None,
    }
}

async fn read_field_bytes_limited(
    mut field: Field<'_>,
    max_bytes: usize,
) -> Result<Vec<u8>, Response> {
    let mut data = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(|_| bad_request())? {
        if data.len() + chunk.len() > max_bytes {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                "مقدار فیلد بیش از حد مجاز است.".to_string(),
            ));
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

async fn read_field_text_limited(field: Field<'_>, max_bytes: usize) -> Result<String, Response> {
    let data = read_field_bytes_limited(field, max_bytes).await?;
    String::from_utf8(data).map_err(|_| bad_request())
}

async fn stream_file_field_to_temp(
    mut field: Field<'_>,
    max_bytes: u64,
    temp_dir: PathBuf,
) -> Result<(PathBuf, u64), Response> {
    fs::create_dir_all(&temp_dir)
        .await
        .map_err(|_| bad_request())?;

    let temp_path = temp_dir.join(format!("{}.part", Uuid::new_v4()));
    let mut file = fs::File::create(&temp_path)
        .await
        .map_err(|_| bad_request())?;

    let mut total: u64 = 0;
    while let Some(chunk) = field.chunk().await.map_err(|_| bad_request())? {
        let chunk_len = chunk.len() as u64;
        if total.saturating_add(chunk_len) > max_bytes {
            let _ = fs::remove_file(&temp_path).await;
            return Err(json_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                "حجم فایل بیش از حد مجاز است.".to_string(),
            ));
        }
        file.write_all(&chunk)
            .await
            .map_err(|_| bad_request())?;
        total += chunk_len;
    }

    if total == 0 {
        let _ = fs::remove_file(&temp_path).await;
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "فایل خالی مجاز نیست.".to_string(),
        ));
    }

    Ok((temp_path, total))
}

pub async fn parse_create_multipart(
    mut multipart: Multipart,
) -> Result<CreateShareInput, Response> {
    let file_upload = app_state().file_upload.clone();
    let max_file_bytes = file_upload.max_size_bytes();
    let upload_temp_dir = crate::features::storage::backend::upload_temp_dir(
        &file_upload.storage.local_root,
    );

    let mut note: Option<String> = None;
    let mut expiry: Option<Expiry> = None;
    let mut max_views: Option<MaxViews> = None;
    let mut one_time_use: Option<bool> = None;
    let mut restrict_ip: Option<bool> = None;
    let mut require_password: Option<bool> = None;
    let mut password: Option<String> = None;
    let mut ip: Option<String> = None;
    let mut file: Option<UploadedFile> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "note" => {
                let text = read_field_text_limited(field, MAX_NOTE_LEN).await?;
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    note = Some(trimmed);
                }
            }
            "expiry" => {
                let text = read_field_text_limited(field, MAX_TEXT_FIELD_LEN).await?;
                expiry = parse_expiry(Some(&text));
            }
            "max_views" => {
                let text = read_field_text_limited(field, MAX_TEXT_FIELD_LEN).await?;
                max_views = parse_max_views(Some(&text));
            }
            "one_time_use" => {
                let text = read_field_text_limited(field, MAX_TEXT_FIELD_LEN).await?;
                one_time_use = parse_bool(Some(&text));
            }
            "restrict_ip" => {
                let text = read_field_text_limited(field, MAX_TEXT_FIELD_LEN).await?;
                restrict_ip = parse_bool(Some(&text));
            }
            "require_password" => {
                let text = read_field_text_limited(field, MAX_TEXT_FIELD_LEN).await?;
                require_password = parse_bool(Some(&text));
            }
            "password" => {
                let text = read_field_text_limited(field, MAX_PASSWORD_LEN).await?;
                if !text.trim().is_empty() {
                    password = Some(text);
                }
            }
            "ip" => {
                let text = read_field_text_limited(field, MAX_IP_FIELD_LEN).await?;
                if !text.trim().is_empty() {
                    ip = Some(text);
                }
            }
            "file" => {
                let filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "upload.bin".to_string());
                let (temp_path, size) =
                    stream_file_field_to_temp(field, max_file_bytes, upload_temp_dir.clone())
                        .await?;
                file = Some(UploadedFile {
                    original_name: filename,
                    temp_path,
                    size,
                });
            }
            _ => {}
        }
    }

    if file.is_some() && !app_state().file_upload.is_upload_available() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "آپلود فایل غیرفعال است.".to_string(),
        ));
    }

    let form = ShareForm {
        note,
        expiry,
        max_views,
        viewed: None,
        downloaded: None,
        one_time_use,
        restrict_ip,
        require_password,
        password,
        ip,
        token: None,
        file_stored_name: None,
        file_original_name: None,
        file_size: None,
    };

    Ok(CreateShareInput { form, file })
}

fn bad_request() -> Response {
    json_error(
        StatusCode::BAD_REQUEST,
        "درخواست نامعتبر است.".to_string(),
    )
}
