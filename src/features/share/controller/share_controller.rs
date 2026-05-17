use crate::features::home::controller::{ErrorTemplate, ShowTemplate};
use crate::features::share::payload::share_delete_payload::DeletePayload;
use crate::features::share::payload::share_show_payload::ShowQuery;
use crate::features::share::service::platform_stats::record_file_downloaded;
use crate::features::share::service::share_service::ShareService;
use crate::features::share::utility::get_client_ip::get_client_ip;
use crate::features::share::utility::parse_create_form::parse_create_multipart;
use crate::features::share::validation::share_delete::DeleteRequest;
use crate::features::share::validation::share_show::ShowRequest;
use crate::features::storage::config::{FileDownloadMode, FileUploadConfig};
use crate::features::storage::ratelimit::FileRateLimiter;
use crate::features::storage::service::StorageService;
use crate::utility::content_disposition::attachment_disposition;
use crate::utility::state::app_state;
use crate::utility::url::append_password_query;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Multipart};
use mime_guess::from_path;
use std::net::SocketAddr;

pub struct ShareController;

impl ShareController {
    /// POST /share
    /// Create a new share
    pub async fn create(
        ConnectInfo(peer): ConnectInfo<SocketAddr>,
        headers: HeaderMap,
        multipart: Multipart,
    ) -> Response {
        let client_ip = get_client_ip(peer.ip(), &headers);
        let input = match parse_create_multipart(multipart).await {
            Ok(i) => i,
            Err(r) => return r,
        };
        ShareService::create(input.form, input.file, Some(client_ip)).await
    }

    /// GET /c/:key — view a share
    pub async fn show(
        ConnectInfo(peer): ConnectInfo<SocketAddr>,
        Path(key): Path<String>,
        Query(query): Query<ShowQuery>,
        headers: HeaderMap,
    ) -> Response {
        let context = crate::features::home::controller::BaseContext::new();
        let client_ip = get_client_ip(peer.ip(), &headers);

        if let Err(limit) = FileRateLimiter::from_state()
            .check_show(Some(client_ip))
            .await
        {
            return ErrorTemplate {
                version: context.version.clone(),
                url: context.url.clone(),
                code: "429".to_string(),
                title: "درخواست زیاد".to_string(),
                message: format!(
                    "تعداد درخواست‌های شما بیش از حد مجاز است. لطفاً {} ثانیه دیگر تلاش کنید.",
                    limit.retry_after_secs
                ),
            }
            .into_response_with_status(StatusCode::TOO_MANY_REQUESTS);
        }

        let id = key.clone();
        let download_password = query.password.clone();
        let request = ShowRequest {
            key,
            password: query.password,
            ip: client_ip.to_string(),
        };

        match ShareService::show(request).await {
            Ok(share_form) => {
                let has_note = share_form
                    .note
                    .as_ref()
                    .map(|n| !n.is_empty())
                    .unwrap_or(false);
                let note = share_form.note.unwrap_or_default();

                let force_proxy = FileUploadConfig::file_requires_proxy_download(
                    share_form.require_password.unwrap_or(false),
                    share_form.restrict_ip.unwrap_or(false),
                    share_form.one_time_use.unwrap_or(false),
                );
                let uses_proxy =
                    force_proxy || app_state().file_upload.download_mode == FileDownloadMode::Proxy;

                let (has_file, file_download_url, file_display_name) =
                    if let (Some(stored), Some(original)) = (
                        share_form.file_stored_name.as_ref(),
                        share_form.file_original_name.as_ref(),
                    ) {
                        let url = StorageService::from_state()
                            .ok()
                            .and_then(|s| s.build_download_url(stored, &id, force_proxy))
                            .map(|mut url| {
                                if uses_proxy {
                                    if let Some(ref pwd) = download_password {
                                        if !pwd.is_empty() {
                                            url = append_password_query(&url, pwd);
                                        }
                                    }
                                }
                                url
                            })
                            .unwrap_or_default();
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

    fn download_error(template: ErrorTemplate) -> Response {
        let status = ErrorTemplate::status_from_code(&template.code);
        template.into_response_with_status(status)
    }

    /// GET /c/:key/file — proxy file download
    pub async fn download_file(
        ConnectInfo(peer): ConnectInfo<SocketAddr>,
        Path(key): Path<String>,
        Query(query): Query<ShowQuery>,
        headers: HeaderMap,
    ) -> Response {
        let context = crate::features::home::controller::BaseContext::new();
        let client_ip = get_client_ip(peer.ip(), &headers);

        if let Err(limit) = FileRateLimiter::from_state()
            .check_download(Some(client_ip), &key)
            .await
        {
            return Self::download_error(ErrorTemplate {
                version: context.version.clone(),
                url: context.url.clone(),
                code: "429".to_string(),
                title: "درخواست زیاد".to_string(),
                message: format!(
                    "تعداد دانلودهای شما بیش از حد مجاز است. لطفاً {} ثانیه دیگر تلاش کنید.",
                    limit.retry_after_secs
                ),
            });
        }

        let request = ShowRequest {
            key: key.clone(),
            password: query.password.clone(),
            ip: client_ip.to_string(),
        };

        let share = match ShareService::access_file_download(&request).await {
            Ok(s) => s,
            Err(error) => {
                if error.code == "401" {
                    return crate::features::home::controller::ShowPasswordTemplate {
                        version: context.version,
                        url: context.url,
                        message: error.message.to_string().parse().unwrap_or(
                            "برای دانلود فایل وارد کردن پسورد الزامی می باشد.".to_string(),
                        ),
                    }
                    .into_response();
                }
                return Self::download_error(ErrorTemplate {
                    version: context.version.clone(),
                    url: context.url.clone(),
                    code: error.code,
                    title: error.title,
                    message: error.message,
                });
            }
        };

        let Some(stored) = share.file_stored_name else {
            return Self::download_error(ErrorTemplate {
                version: context.version,
                url: context.url,
                code: "404".to_string(),
                title: "یافت نشد".to_string(),
                message: "فایلی برای این اشتراک وجود ندارد.".to_string(),
            });
        };

        let original = share
            .file_original_name
            .as_deref()
            .unwrap_or("download.bin");

        let storage = match StorageService::from_state() {
            Ok(s) => s,
            Err(_) => {
                return Self::download_error(ErrorTemplate {
                    version: context.version.clone(),
                    url: context.url.clone(),
                    code: "500".to_string(),
                    title: "خطای سرور".to_string(),
                    message: "خطا در دسترسی به فایل.".to_string(),
                });
            }
        };

        let data = match storage.get_stored(&stored).await {
            Ok(d) => d,
            Err(_) => {
                return Self::download_error(ErrorTemplate {
                    version: context.version,
                    url: context.url,
                    code: "404".to_string(),
                    title: "یافت نشد".to_string(),
                    message: "فایل یافت نشد.".to_string(),
                });
            }
        };

        let downloaded_bytes = share.file_size.unwrap_or(data.len() as u64);
        if let Err(e) = record_file_downloaded(downloaded_bytes).await {
            tracing::warn!(target: "system", "Failed to record download stats: {}", e);
        }

        let mime = from_path(original).first_or_octet_stream();
        let disposition = attachment_disposition(&StorageService::display_filename(original));

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&disposition).unwrap_or_else(|_| {
                    HeaderValue::from_static("attachment")
                }),
            )
            .body(Body::from(data))
            .unwrap()
            .into_response()
    }

    /// DELETE /api/share/:key
    pub async fn delete(Path(key): Path<String>, Json(payload): Json<DeletePayload>) -> Response {
        let request = DeleteRequest {
            key,
            token: payload.token,
        };

        ShareService::delete(request).await
    }
}
