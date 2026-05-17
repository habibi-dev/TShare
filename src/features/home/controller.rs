use crate::features::share::service::used_count::get_used_count;
use crate::utility::state::app_state;
use crate::utility::url::url;
use askama::Template;
use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};

// Base context shared across all templates
pub(crate) struct BaseContext {
    pub(crate) version: String,
    pub(crate) url: String,
}

impl BaseContext {
    pub(crate) fn new() -> Self {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        Self {
            version: VERSION.into(),
            url: url(""),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    version: String,
    url: String,
    used: String,
    file_upload_enabled: bool,
    note_required: bool,
    file_max_size_mb: u64,
    file_accept_attr: String,
    file_allowed_extensions_display: String,
}

#[derive(Template)]
#[template(path = "history.html")]
struct HistoryTemplate {
    version: String,
    url: String,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub version: String,
    pub url: String,
    pub code: String,
    pub title: String,
    pub message: String,
}

impl ErrorTemplate {
    pub fn into_response_with_status(self, status: StatusCode) -> Response {
        match self.render() {
            Ok(html) => Response::builder()
                .status(status)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(header::CONTENT_DISPOSITION, "inline")
                .body(Body::from(html))
                .unwrap()
                .into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render template",
            )
                .into_response(),
        }
    }

    pub fn status_from_code(code: &str) -> StatusCode {
        match code {
            "401" => StatusCode::UNAUTHORIZED,
            "403" => StatusCode::FORBIDDEN,
            "404" => StatusCode::NOT_FOUND,
            "410" => StatusCode::GONE,
            "429" => StatusCode::TOO_MANY_REQUESTS,
            "500" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Template)]
#[template(path = "show.html")]
pub struct ShowTemplate {
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) count: String,
    pub(crate) note: String,
    pub(crate) id: String,
    pub(crate) has_note: bool,
    pub(crate) has_file: bool,
    pub(crate) file_download_url: String,
    pub(crate) file_display_name: String,
}

#[derive(Template)]
#[template(path = "show-password.html")]
pub struct ShowPasswordTemplate {
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) message: String,
}

pub struct HomeController;

impl HomeController {
    pub async fn index() -> impl IntoResponse {
        let context = BaseContext::new();
        let count = get_used_count().await.unwrap_or(0);

        let file_cfg = &app_state().file_upload;
        IndexTemplate {
            version: context.version,
            url: context.url,
            used: count.to_string(),
            file_upload_enabled: file_cfg.is_upload_available(),
            note_required: !file_cfg.is_upload_available(),
            file_max_size_mb: file_cfg.max_size_mb,
            file_accept_attr: file_cfg.accept_attr(),
            file_allowed_extensions_display: file_cfg.extensions_display(),
        }
    }

    pub async fn history() -> impl IntoResponse {
        let context = BaseContext::new();
        HistoryTemplate {
            version: context.version,
            url: context.url,
        }
    }

    pub async fn not_found() -> impl IntoResponse {
        let context = BaseContext::new();
        ErrorTemplate {
            version: context.version,
            url: context.url,
            code: "404".to_string(),
            title: "صفحه مورد نظر یافت نشد".to_string(),
            message: "ممکن است لینک اشتباه باشد یا محتوا حذف شده باشد.".to_string(),
        }
    }
}

// Macro to implement IntoResponse for template structs
macro_rules! impl_template_response {
    ($($template:ty),+ $(,)?) => {
        $(
            impl IntoResponse for $template {
                fn into_response(self) -> Response {
                    match self.render() {
                        Ok(html) => Html(html).into_response(),
                        Err(_) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to render template",
                        )
                            .into_response(),
                    }
                }
            }
        )+
    };
}

impl_template_response!(
    IndexTemplate,
    HistoryTemplate,
    ErrorTemplate,
    ShowTemplate,
    ShowPasswordTemplate
);
