use crate::features::share::validation::share_form::{Expiry, MaxViews};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub note: String,
    pub expiry: Expiry,
    pub max_views: MaxViews,
    pub one_time_use: bool,
    pub restrict_ip: bool,
    pub require_password: bool,
    pub password: Option<String>,
    pub ip: String,
}
