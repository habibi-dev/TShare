use crate::features::share::service::share_create::ShareCreate;
use crate::features::share::service::share_delete::ShareDelete;
use crate::features::share::service::share_retrieve::{ShareError, ShareRetrieve};
use crate::features::share::validation::share_delete::DeleteRequest;
use crate::features::share::validation::share_form::ShareForm;
use crate::features::share::validation::share_show::ShowRequest;
use crate::features::storage::service::UploadedFile;
use axum::response::Response;
use std::net::IpAddr;

pub struct ShareService;

impl ShareService {
    pub async fn create(
        form: ShareForm,
        file: Option<UploadedFile>,
        client_ip: Option<IpAddr>,
    ) -> Response {
        *ShareCreate::execute(form, file, client_ip).await
    }

    pub async fn show(request: ShowRequest) -> Result<ShareForm, ShareError> {
        ShareRetrieve::execute(request).await
    }

    pub async fn authorize_file_download(
        request: &ShowRequest,
    ) -> Result<ShareForm, ShareError> {
        ShareRetrieve::authorize_download(request).await
    }

    pub async fn consume_file_download(request: &ShowRequest) -> Result<(), ShareError> {
        ShareRetrieve::consume_download(request).await
    }

    // pub async fn update(request: UpdateRequest) -> Response {
    //     *ShareUpdate::execute(request).await
    // }
    //
    pub async fn delete(request: DeleteRequest) -> Response {
        *ShareDelete::execute(request).await
    }
}
