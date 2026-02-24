use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ShowQuery {
    pub password: Option<String>,
}
