use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteRequest {
    #[validate(length(min = 1))]
    pub key: String,

    #[validate(length(min = 1))]
    pub token: String,
}
