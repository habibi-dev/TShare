/*use crate::features::share::validation::share_form::{Expiry, MaxViews};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePayload {
    #[validate(length(min = 1))]
    pub token: String,

    #[validate(length(max = 10000))]
    pub note: Option<String>,

    pub max_views: Option<MaxViews>,

    pub one_time_use: Option<bool>,

    pub require_password: Option<bool>,

    pub password: Option<String>,
    pub restrict_ip: Option<bool>,
    #[validate(ip)]
    pub ip: Option<String>,
    pub expiry: Option<Expiry>,
}
*/
