use crate::core::app_error::{handle_normalize_error, handle_tower_error};
use crate::core::logger::targets;
use crate::core::state::AppState;
use crate::utility::state::app_state;
use crate::features::home::controller::HomeController;
use axum::error_handling::HandleErrorLayer;
use axum::extract::{Path, Request};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Router as AxumRouter, body::Body, middleware};
use rust_embed::RustEmbed;
use std::time::Duration;
use std::time::Instant;
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(RustEmbed)]
#[folder = "src/assets/"]
struct Assets;

pub struct Router;
impl Router {
    pub fn routes(app_state: AppState, route_lists: Vec<(&str, axum::Router)>) -> AxumRouter {
        let mut route_list = AxumRouter::new();

        for (prefix_raw, routes) in route_lists {
            let prefix = Self::normalize_prefix(prefix_raw);
            // Use merge at root, nest otherwise
            route_list = if prefix.is_empty() || prefix == "/" {
                route_list.merge(routes)
            } else {
                route_list.nest(&prefix, routes)
            };
        }

        AxumRouter::new()
            .with_state(app_state.clone())
            .merge(route_list)
            .route("/{*path}", get(Self::assets))
            .fallback(HomeController::not_found)
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(handle_tower_error))
                    .layer(TimeoutLayer::new(Duration::from_secs(30))) // inner
                    .layer(TraceLayer::new_for_http())
                    .into_inner(),
            )
            .route_layer(middleware::from_fn(handle_normalize_error))
            .route_layer(middleware::from_fn(Self::security_headers))
            .route_layer(middleware::from_fn(Self::log_requests))
    }

    async fn security_headers(req: Request<Body>, next: middleware::Next) -> Response {
        let mut response = next.run(req).await;
        let headers = response.headers_mut();
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
        headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
        headers.insert(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        );
        headers.insert(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        );
        if app_state().config.https {
            headers.insert(
                header::STRICT_TRANSPORT_SECURITY,
                HeaderValue::from_static("max-age=63072000; includeSubDomains"),
            );
        }
        response
    }

    async fn log_requests(req: Request<Body>, next: middleware::Next) -> Response {
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let start = Instant::now();

        let response = next.run(req).await;
        let status = response.status().as_u16();
        let elapsed = start.elapsed().as_millis();

        info!(
            target: targets::REQUEST,
            method,
            path,
            status,
            latency_ms = elapsed,
            "HTTP request completed"
        );

        response
    }

    pub async fn assets(Path(path): Path<String>) -> impl IntoResponse {
        let path = path.trim_start_matches('/');

        if path.contains("..") || path.contains('\\') {
            return HomeController::not_found().await.into_response();
        }

        match Assets::get(path) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(Body::from(content.data.to_vec()))
                    .unwrap()
            }
            None => HomeController::not_found().await.into_response(),
        }
    }

    fn normalize_prefix(raw: &str) -> String {
        let s = raw.trim();
        if s.is_empty() || s == "/" {
            return "/".to_string();
        }
        let s = s.trim_matches('/');
        format!("/{}", s)
    }
}
