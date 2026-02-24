use crate::core::router::Router as MyRouter;
use crate::core::state::AppState;
use crate::features::home::controller::HomeController;
use crate::features::share::routes::{share_api_route, share_route};
use axum::routing::get;
use axum::{Router as AxumRouter, Router};

pub(crate) struct Routes;
impl Routes {
    pub fn generate(app_state: AppState) -> AxumRouter {
        let routers_list: Vec<(&str, Router)> = vec![
            ("/", Router::new().route("/", get(HomeController::index))),
            (
                "/",
                Router::new().route("/history", get(HomeController::history)),
            ),
            share_api_route(),
            share_route(),
        ];

        MyRouter::routes(app_state, routers_list)
    }
}
