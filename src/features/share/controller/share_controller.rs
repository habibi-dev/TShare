use crate::features::home::controller::{ErrorTemplate, ShowTemplate};
use crate::features::share::payload::share_delete_payload::DeletePayload;
use crate::features::share::payload::share_show_payload::ShowQuery;
use crate::features::share::service::share_service::ShareService;
use crate::features::share::utility::get_client_ip::get_client_ip;
use crate::features::share::validation::share_delete::DeleteRequest;
use crate::features::share::validation::share_form::ShareForm;
use crate::features::share::validation::share_show::ShowRequest;
use axum::extract::{Path, Query};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};

pub struct ShareController;

impl ShareController {
    /// POST /share
    /// Create a new share
    pub async fn create(Form(form): Form<ShareForm>) -> Response {
        ShareService::create(form).await
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
                ShowTemplate {
                    version: context.version,
                    url: context.url,
                    count: share_form.viewed.unwrap_or(0).to_string(),
                    id,
                    note: share_form.note.unwrap(),
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
