//! Request classification: content-type/stream detection.
//! Pure functions over headers and the request path.

use super::headers::header_contains;
use axum::http::HeaderMap;

#[allow(dead_code)]
pub(super) fn is_event_stream(headers: &HeaderMap) -> bool {
    header_contains(headers, http::header::CONTENT_TYPE, "text/event-stream")
}

#[allow(dead_code)]
pub(super) fn declared_content_length(headers: &HeaderMap) -> Option<usize> {
    headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
}

// gRPC and Dubbo Triple mark themselves via content-type and only run over
// HTTP/2. grpc-web is excluded on purpose: it is designed to cross HTTP/1
// intermediaries with trailers encoded in the body.
pub(super) fn is_grpc_request(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let content_type = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();
    content_type == "application/grpc"
        || content_type.starts_with("application/grpc+")
        || content_type == "application/triple"
        || content_type.starts_with("application/triple+")
}
