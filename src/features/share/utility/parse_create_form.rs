use crate::core::response::json_error;
use crate::features::share::validation::share_form::{Expiry, MaxViews, ShareForm};
use crate::features::storage::service::UploadedFile;
use crate::utility::state::app_state;
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::Response;

pub struct CreateShareInput {
    pub form: ShareForm,
    pub file: Option<UploadedFile>,
}

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

pub async fn parse_create_multipart(
    mut multipart: Multipart,
) -> Result<CreateShareInput, Response> {
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
                let text = field.text().await.map_err(|_| bad_request())?;
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    note = Some(trimmed);
                }
            }
            "expiry" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                expiry = parse_expiry(Some(&text));
            }
            "max_views" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                max_views = parse_max_views(Some(&text));
            }
            "one_time_use" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                one_time_use = parse_bool(Some(&text));
            }
            "restrict_ip" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                restrict_ip = parse_bool(Some(&text));
            }
            "require_password" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                require_password = parse_bool(Some(&text));
            }
            "password" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                if !text.trim().is_empty() {
                    password = Some(text);
                }
            }
            "ip" => {
                let text = field.text().await.map_err(|_| bad_request())?;
                if !text.trim().is_empty() {
                    ip = Some(text);
                }
            }
            "file" => {
                let filename = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "upload.bin".to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| bad_request())?
                    .to_vec();
                if !data.is_empty() {
                    file = Some(UploadedFile {
                        original_name: filename,
                        data: Bytes::from(data),
                    });
                }
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
