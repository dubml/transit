//! Pure header manipulation: hop-by-hop stripping and matching.
//! None of these touch server state; they operate only on a `HeaderMap`.

use axum::http::{HeaderMap, HeaderName};

pub(super) fn remove_connection_headers(headers: &mut HeaderMap) {
    let named: Vec<String> = headers
        .get_all(http::header::CONNECTION)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(|name| name.trim().to_ascii_lowercase())
        .collect();
    for name in named {
        if let Ok(name) = HeaderName::try_from(name.as_str()) {
            headers.remove(name);
        }
    }
    for name in [
        http::header::CONNECTION,
        http::header::PROXY_AUTHENTICATE,
        http::header::PROXY_AUTHORIZATION,
        http::header::TRANSFER_ENCODING,
        http::header::UPGRADE,
    ] {
        headers.remove(name);
    }
    headers.remove(HeaderName::from_static("keep-alive"));
    headers.remove(HeaderName::from_static("proxy-connection"));
    headers.remove(HeaderName::from_static("http2-settings"));
}

#[allow(dead_code)]
pub(super) fn header_contains(headers: &HeaderMap, name: HeaderName, needle: &str) -> bool {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains(needle))
        .unwrap_or(false)
}
