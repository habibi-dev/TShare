use serde::{Deserialize, Serialize};
use validator::Validate;
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ShareForm {
    pub note: Option<String>,
    pub expiry: Option<Expiry>,
    pub max_views: Option<MaxViews>,
    pub viewed: Option<u64>,
    /// Set after a one-time file share is downloaded (page views use `viewed`).
    pub downloaded: Option<bool>,
    pub one_time_use: Option<bool>,
    pub restrict_ip: Option<bool>,
    pub require_password: Option<bool>,
    pub password: Option<String>,
    pub ip: Option<String>,
    pub token: Option<String>,
    pub file_stored_name: Option<String>,
    pub file_original_name: Option<String>,
    pub file_size: Option<u64>,
}

impl ShareForm {
    pub fn has_content(&self) -> bool {
        self.note
            .as_ref()
            .map(|n| !n.trim().is_empty())
            .unwrap_or(false)
            || self.file_stored_name.is_some()
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Expiry {
    #[serde(alias = "5")]
    N5,
    #[serde(alias = "10")]
    N10,
    #[serde(alias = "30")]
    N30,
    #[serde(alias = "60")]
    N60,
    #[serde(alias = "180")]
    N180,
    #[serde(alias = "360")]
    N360,
    #[serde(alias = "720")]
    N720,
    #[serde(alias = "1440")]
    N1440,
}

impl Expiry {
    pub fn as_seconds(&self) -> u64 {
        match *self {
            Expiry::N5 => 5 * 60,
            Expiry::N10 => 10 * 60,
            Expiry::N30 => 30 * 60,
            Expiry::N60 => 60 * 60,
            Expiry::N180 => 180 * 60,
            Expiry::N360 => 360 * 60,
            Expiry::N720 => 720 * 60,
            Expiry::N1440 => 1440 * 60,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MaxViews {
    #[serde(alias = "2")]
    V1,
    #[serde(alias = "6")]
    V5,
    #[serde(alias = "11")]
    V10,
    #[serde(alias = "unlimited")]
    UNLIMITED,
}

impl MaxViews {
    pub fn as_int(&self) -> i32 {
        match *self {
            MaxViews::V1 => 2,
            MaxViews::V5 => 6,
            MaxViews::V10 => 11,
            MaxViews::UNLIMITED => -1,
        }
    }
}
