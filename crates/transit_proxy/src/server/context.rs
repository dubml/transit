//! The parsed per-request agent context: protocol, routing keys, and HTTP headers.

use super::routing::{header_pairs, host_header};
use axum::http::{HeaderMap, HeaderName, HeaderValue as HttpHeaderValue, Method};
use hyper::body::Bytes;
use transit::{AgentMatchInput, AgentProtocol};

#[derive(Debug, Clone)]
pub(super) struct AgentRequestContext {
    pub(super) protocol: AgentProtocol,
    pub(super) host: String,
    pub(super) path: String,
    pub(super) path_and_query: String,
    pub(super) method: Method,
    pub(super) stream_hint: bool,
    pub(super) headers: Vec<(String, String)>,
}

impl AgentRequestContext {
    pub(super) fn new(protocol: AgentProtocol, parts: &http::request::Parts, _body: &Bytes) -> Self {
        let host = host_header(&parts.headers).unwrap_or("*").to_string();
        let path = parts.uri.path().to_string();
        let path_and_query = parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| path.clone());

        Self {
            protocol,
            host,
            path,
            path_and_query,
            method: parts.method.clone(),
            stream_hint: false,
            headers: header_pairs(&parts.headers),
        }
    }

    pub(super) fn input(&self) -> AgentMatchInput<'_> {
        AgentMatchInput {
            protocol: self.protocol,
            host: &self.host,
            path: &self.path,
            method: self.method.as_str(),
            model: None,
            tool: None,
            agent: None,
            headers: &self.headers,
        }
    }
}

pub(super) fn apply_stream_headers(headers: &mut HeaderMap, context: &AgentRequestContext) {
    if !context.stream_hint {
        return;
    }
    headers
        .entry(http::header::CACHE_CONTROL)
        .or_insert_with(|| HttpHeaderValue::from_static("no-cache, no-transform"));
    headers.insert(
        HeaderName::from_static("x-accel-buffering"),
        HttpHeaderValue::from_static("no"),
    );
}
