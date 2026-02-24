use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DeletePayload {
    #[validate(length(min = 1))]
    pub token: String,
}
