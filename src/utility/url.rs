use crate::utility::state::app_state;

pub fn url(path: &str) -> String {
    let config = app_state().config.clone();
    let domain = config.final_domain;
    let https = config.https;

    let scheme = if https { "https" } else { "http" };

    let clean_path = path.trim_start_matches('/');
    format!("{scheme}://{domain}/{}", clean_path)
}

/// Appends `?password=` or `&password=` with percent-encoded value.
pub fn append_password_query(target_url: &str, password: &str) -> String {
    let encoded = password
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect::<String>();
    let separator = if target_url.contains('?') { '&' } else { '?' };
    format!("{target_url}{separator}password={encoded}")
}
