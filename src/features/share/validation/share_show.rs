use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ShowRequest {
    pub key: String,
    pub password: Option<String>,
    pub ip: String,
}
