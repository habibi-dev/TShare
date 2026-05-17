use crate::utility::state::app_state;
use axum::http::HeaderMap;
use std::net::IpAddr;

fn ip_from_forwarded_headers(headers: &HeaderMap) -> Option<IpAddr> {
    if let Some(val) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        && let Some(first) = val.split(',').next()
        && let Ok(ip) = first.trim().parse::<IpAddr>()
    {
        return Some(ip);
    }

    if let Some(val) = headers.get("x-real-ip").and_then(|v| v.to_str().ok())
        && let Ok(ip) = val.trim().parse::<IpAddr>()
    {
        return Some(ip);
    }

    None
}

/// Returns the client IP. Forwarded headers are trusted only when the TCP peer
/// is listed in `TRUSTED_PROXIES` (e.g. your reverse proxy).
pub fn get_client_ip(peer: IpAddr, headers: &HeaderMap) -> IpAddr {
    let trusted = &app_state().config.trusted_proxies;
    if trusted.contains(&peer)
        && let Some(ip) = ip_from_forwarded_headers(headers)
    {
        return ip;
    }
    peer
}
