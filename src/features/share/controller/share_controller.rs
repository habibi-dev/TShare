use crate::features::home::controller::{ErrorTemplate, ShowTemplate};
use crate::features::share::payload::share_delete_payload::DeletePayload;
use crate::features::share::payload::share_show_payload::ShowQuery;
use crate::features::share::service::share_service::ShareService;
use crate::features::share::utility::get_client_ip::get_client_ip;
use crate::features::share::utility::parse_create_form::parse_create_multipart;
use crate::features::share::validation::share_delete::DeleteRequest;
use crate::features::share::validation::share_show::ShowRequest;
use crate::features::storage::config::FileDownloadMode;
use crate::features::storage::service::StorageService;
use crate::utility::state::app_state;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Multipart};
use mime_guess::from_path;

pub struct ShareController;

impl ShareController {
    /// POST /share
    /// Create a new share
    pub async fn create(multipart: Multipart) -> Response {
        let input = match parse_create_multipart(multipart).await {
            Ok(i) => i,
            Err(r) => return r,
        };
        ShareService::create(input.form, input.file).await
    }

    /// GET /share/:key
    /// Retrieve and view a share
    pub async fn show(
        Path(key): Path<String>,
        Query(query): Query<ShowQuery>,
        headers: HeaderMap,
    ) -> Response {
        let id = key.clone();
        let request = ShowRequest {
            key,
            password: query.password,
            ip: get_client_ip(&headers)
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        };

        match ShareService::show(request).await {
            Ok(share_form) => {
                let context = crate::features::home::controller::BaseContext::new();
                let has_note = share_form
                    .note
                    .as_ref()
                    .map(|n| !n.is_empty())
                    .unwrap_or(false);
                let note = share_form.note.unwrap_or_default();

                let (has_file, file_download_url, file_display_name) =
                    if let (Some(stored), Some(original)) = (
                        share_form.file_stored_name.as_ref(),
                        share_form.file_original_name.as_ref(),
                    ) {
                        let url = if app_state().file_upload.download_mode
                            == FileDownloadMode::Proxy
                        {
                            StorageService::from_state()
                                .ok()
                                .and_then(|s| s.build_download_url(stored, &id))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        (
                            true,
                            url,
                            StorageService::display_filename(original),
                        )
                    } else {
                        (false, String::new(), String::new())
                    };

                ShowTemplate {
                    version: context.version,
                    url: context.url,
                    count: share_form.viewed.unwrap_or(0).to_string(),
                    id,
                    note,
                    has_note,
                    has_file,
                    file_download_url,
                    file_display_name,
                }
                .into_response()
            }
            Err(error) => {
                let context = crate::features::home::controller::BaseContext::new();

                if error.code == "401" {
                    return crate::features::home::controller::ShowPasswordTemplate {
                        version: context.version,
                        url: context.url,
                        message: error.message.to_string().parse().unwrap_or(
                            "برای مشاهده این متن وارد کردن پسورد الزامی می باشد.".to_string(),
                        ),
                    }
                    .into_response();
                }

                ErrorTemplate {
                    version: context.version,
                    url: context.url,
                    code: error.code,
                    title: error.title,
                    message: error.message,
                }
                .into_response()
            }
        }
    }

    // PUT /share/:key
    // Update an existing share
    /*pub async fn update(Path(key): Path<String>, Json(payload): Json<UpdatePayload>) -> Response {
            let request = UpdateRequest {
                key,
                token: payload.token,
                note: payload.note,
                expiry: payload.expiry,
                max_views: payload.max_views,
                one_time_use: payload.one_time_use,
                restrict_ip: payload.restrict_ip,
                require_password: payload.require_password,
                password: payload.password,
                ip: payload.ip,
            };

            ShareService::update(request).await
        }
    */
    /// GET /c/:key/file — proxy file download (when FILE_DOWNLOAD_MODE=proxy)
    pub async fn download_file(
        Path(key): Path<String>,
        Query(query): Query<ShowQuery>,
        headers: HeaderMap,
    ) -> Response {
        if app_state().file_upload.download_mode != FileDownloadMode::Proxy {
            return ErrorTemplate {
                version: crate::features::home::controller::BaseContext::new().version,
                url: crate::features::home::controller::BaseContext::new().url,
                code: "404".to_string(),
                title: "یافت نشد".to_string(),
                message: "دانلود فایل از این مسیر پشتیبانی نمی‌شود.".to_string(),
            }
            .into_response();
        }

        let request = crate::features::share::validation::share_show::ShowRequest {
            key: key.clone(),
            password: query.password,
            ip: get_client_ip(&headers)
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        };

        let share = match ShareService::authorize_file_download(request).await {
            Ok(s) => s,
            Err(error) => {
                let context = crate::features::home::controller::BaseContext::new();
                return ErrorTemplate {
                    version: context.version,
                    url: context.url,
                    code: error.code,
                    title: error.title,
                    message: error.message,
                }
                .into_response();
            }
        };

        let Some(stored) = share.file_stored_name else {
            let context = crate::features::home::controller::BaseContext::new();
            return ErrorTemplate {
                version: context.version,
                url: context.url,
                code: "404".to_string(),
                title: "یافت نشد".to_string(),
                message: "فایلی برای این اشتراک وجود ندارد.".to_string(),
            }
            .into_response();
        };

        let original = share
            .file_original_name
            .as_deref()
            .unwrap_or("download.bin");

        let storage = match StorageService::from_state() {
            Ok(s) => s,
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let data = match storage.get_stored(&stored).await {
            Ok(d) => d,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };

        let mime = from_path(original).first_or_octet_stream();
        let disposition = format!(
            "attachment; filename=\"{}\"",
            StorageService::display_filename(original)
        );

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&disposition).unwrap_or_else(|_| {
                    HeaderValue::from_static("attachment")
                }),
            )
            .body(Body::from(data.to_vec()))
            .unwrap()
            .into_response()
    }

    // DELETE /share/:key
    // Delete a share
    pub async fn delete(Path(key): Path<String>, Json(payload): Json<DeletePayload>) -> Response {
        let request = DeleteRequest {
            key,
            token: payload.token,
        };

        ShareService::delete(request).await
    }
}
