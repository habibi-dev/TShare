use crate::features::share::controller::share_controller::ShareController;
use crate::utility::state::app_state;
use axum::Router;
use axum::routing::{delete, get, post};

pub fn share_api_route() -> (&'static str, Router) {
    let state = app_state();

    (
        "api",
        Router::new()
            .route("/share", post(ShareController::create))
            // .route("/share/{key}", put(ShareController::update))
            .route("/share/{key}", delete(ShareController::delete))
            .with_state(state),
    )
}

pub fn share_route() -> (&'static str, Router) {
    let state = app_state();

    (
        "/",
        Router::new()
            .route("/c/{code}", get(ShareController::show))
            .with_state(state),
    )
}
