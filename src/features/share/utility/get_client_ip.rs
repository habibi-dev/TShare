use axum::http::HeaderMap;
use std::net::IpAddr;

pub fn get_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // X-Forwarded-For: take the first IP
    if let Some(val) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        && let Some(first) = val.split(',').next()
        && let Ok(ip) = first.trim().parse::<IpAddr>()
    {
        return Some(ip);
    }

    // X-Real-IP
    if let Some(val) = headers.get("x-real-ip").and_then(|v| v.to_str().ok())
        && let Ok(ip) = val.trim().parse::<IpAddr>()
    {
        return Some(ip);
    }

    None
}
