use crate::features::share::service::used_count::get_used_count;
use crate::utility::url::url;
use askama::Template;
use axum::http::StatusCode;
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

#[derive(Template)]
#[template(path = "show.html")]
pub struct ShowTemplate {
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) note: String,
    pub(crate) id: String,
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

        IndexTemplate {
            version: context.version,
            url: context.url,
            used: count.to_string(),
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
