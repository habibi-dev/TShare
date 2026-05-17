/// Safe `Content-Disposition: attachment` value with RFC 5987 `filename*`.
pub fn attachment_disposition(display_name: &str) -> String {
    let safe: String = display_name
        .chars()
        .filter(|c| !c.is_control() && *c != '"' && *c != '\\' && *c != '/')
        .take(200)
        .collect();

    let fallback = if safe.is_empty() {
        "download.bin".to_string()
    } else {
        safe.clone()
    };

    let encoded: String = safe
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();

    format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{encoded}",
        fallback.replace('"', "_")
    )
}
