use crate::features::share::validation::share_form::{Expiry, MaxViews};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRequest {
    #[validate(length(min = 1))]
    pub key: String,
    #[validate(required, length(max = 10000))]
    pub note: Option<String>,
    pub expiry: Option<Expiry>,
    pub max_views: Option<MaxViews>,
    pub one_time_use: Option<bool>,
    pub restrict_ip: Option<bool>,
    pub require_password: Option<bool>,
    pub password: Option<String>,
    #[validate(ip)]
    pub ip: Option<String>,
    #[validate(length(min = 1))]
    pub token: String,
}
