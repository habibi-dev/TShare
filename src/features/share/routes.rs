use crate::core::response::json_error;
use crate::features::share::controller::share_controller::ShareController;
use crate::utility::state::app_state;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{delete, get, post};
use std::time::Duration;
use tokio::time::timeout;

fn upload_body_limit_bytes() -> usize {
    let cfg = &app_state().file_upload;
    let file_bytes = cfg.max_size_bytes() as usize;
    file_bytes.saturating_add(512 * 1024)
}

fn upload_timeout_duration() -> Duration {
    let secs = std::env::var("FILE_UPLOAD_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|s| *s > 0)
        .unwrap_or(300);
    Duration::from_secs(secs)
}

async fn upload_request_timeout(req: Request, next: Next) -> Response {
    match timeout(upload_timeout_duration(), next.run(req)).await {
        Ok(response) => response,
        Err(_) => json_error(
            StatusCode::REQUEST_TIMEOUT,
            "زمان آپلود به پایان رسید. لطفاً فایل کوچک‌تری انتخاب کنید یا دوباره تلاش کنید.".to_string(),
        ),
    }
}

pub fn share_api_route() -> (&'static str, Router) {
    let state = app_state();

    let upload_routes = Router::new()
        .route(
            "/share",
            post(ShareController::create).layer(DefaultBodyLimit::max(upload_body_limit_bytes())),
        )
        .layer(middleware::from_fn(upload_request_timeout));

    (
        "api",
        Router::new()
            .merge(upload_routes)
            .route("/share/{key}", delete(ShareController::delete))
            .with_state(state),
    )
}

pub fn share_route() -> (&'static str, Router) {
    let state = app_state();

    (
        "/",
        Router::new()
            .route("/c/{code}/file", get(ShareController::download_file))
            .route("/c/{code}", get(ShareController::show))
            .with_state(state),
    )
}
