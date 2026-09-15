//! Pure routing/backend helpers: provider lookup, protocol matching, upstream
//! URI composition, TLS-mode classification, and header extraction. These
//! depend only on config types and `HeaderMap`, not on server state.

use axum::http::{HeaderMap, StatusCode, Uri};
use transit_core::{
    AgentProtocol, Backend, BackendKind, ConfigSnapshot, Endpoint, Provider, UpstreamTls,
    UpstreamTlsMode,
};

pub(super) fn backend_provider<'a>(
    snapshot: &'a ConfigSnapshot,
    backend: &'a Backend,
) -> Option<&'a Provider> {
    match &backend.kind {
        BackendKind::Llm { provider, .. } => {
            snapshot.provider(provider).map(|provider| &**provider)
        }
        _ => None,
    }
}

pub(super) fn backend_matches_protocol(backend: &Backend, protocol: AgentProtocol) -> bool {
    matches!(
        (&backend.kind, protocol),
        (BackendKind::Http { .. }, AgentProtocol::Http)
            | (BackendKind::Llm { .. }, AgentProtocol::Llm)
            | (BackendKind::Mcp { .. }, AgentProtocol::Mcp)
            | (BackendKind::A2a { .. }, AgentProtocol::A2a)
    )
}

pub(super) fn compose_upstream_uri(
    endpoint: &str,
    path_and_query: &str,
) -> Result<Uri, (StatusCode, String)> {
    let endpoint = endpoint.trim_end_matches('/');
    let suffix = if endpoint.ends_with("/v1") && path_and_query.starts_with("/v1/") {
        path_and_query.trim_start_matches("/v1")
    } else {
        path_and_query
    };
    format!("{endpoint}{suffix}").parse::<Uri>().map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("invalid upstream uri for endpoint {endpoint}: {e}"),
        )
    })
}

pub(super) fn compose_backend_uri(
    protocol: AgentProtocol,
    endpoint: &str,
    path_and_query: &str,
) -> Result<Uri, (StatusCode, String)> {
    let target = endpoint
        .parse::<Uri>()
        .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid backend endpoint".into()))?;
    // MCP/A2A targets with an explicit path identify the RPC endpoint. The
    // public route path may be different and must not be appended a second time.
    if matches!(protocol, AgentProtocol::Mcp | AgentProtocol::A2a)
        && !(protocol == AgentProtocol::A2a
            && path_and_query.split('?').next() == Some(crate::a2a::AGENT_CARD_PATH))
    {
        let mut path = if target.path().is_empty() || target.path() == "/" {
            path_and_query.split('?').next().unwrap_or("/").to_string()
        } else {
            target.path().to_string()
        };
        let query: Vec<&str> = [
            target.query(),
            path_and_query.split_once('?').map(|(_, query)| query),
        ]
        .into_iter()
        .flatten()
        .filter(|value| !value.is_empty())
        .collect();
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query.join("&"));
        }
        let mut parts = target.into_parts();
        parts.path_and_query = Some(path.parse().map_err(|_| {
            (
                StatusCode::BAD_GATEWAY,
                "invalid backend path or query".into(),
            )
        })?);
        return Uri::from_parts(parts)
            .map_err(|_| (StatusCode::BAD_GATEWAY, "invalid backend endpoint".into()));
    }
    compose_upstream_uri(endpoint, path_and_query)
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;

    #[test]
    fn rpc_endpoint_paths_replace_public_paths_and_preserve_queries() {
        assert_eq!(
            compose_backend_uri(
                AgentProtocol::Mcp,
                "https://backend/mcp?tenant=one",
                "/public/tools?cursor=two"
            )
            .unwrap(),
            "https://backend/mcp?tenant=one&cursor=two"
        );
        assert_eq!(
            compose_backend_uri(AgentProtocol::A2a, "https://backend/rpc", "/agent").unwrap(),
            "https://backend/rpc"
        );
        assert_eq!(
            compose_backend_uri(AgentProtocol::Mcp, "https://backend", "/mcp").unwrap(),
            "https://backend/mcp"
        );
        assert_eq!(
            compose_backend_uri(
                AgentProtocol::Mcp,
                "https://backend/?tenant=one",
                "/mcp?cursor=two"
            )
            .unwrap(),
            "https://backend/mcp?tenant=one&cursor=two"
        );
        assert_eq!(
            compose_backend_uri(AgentProtocol::Http, "https://backend/base", "/public").unwrap(),
            "https://backend/base/public"
        );
    }
}

pub(super) fn protocol_name(protocol: AgentProtocol) -> &'static str {
    match protocol {
        AgentProtocol::Http => "http",
        AgentProtocol::Llm => "llm",
        AgentProtocol::Mcp => "mcp",
        AgentProtocol::A2a => "a2a",
    }
}

pub(super) fn endpoint_authority(endpoint: &Endpoint) -> String {
    if endpoint.address.contains(':') && !endpoint.address.starts_with('[') {
        format!("[{}]:{}", endpoint.address, endpoint.port)
    } else {
        format!("{}:{}", endpoint.address, endpoint.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UpstreamRequestMode {
    PlainHttp,
    SimpleTls,
    DubboMutual,
}

pub(super) fn upstream_request_mode(tls: Option<&UpstreamTls>) -> UpstreamRequestMode {
    match tls.map(|tls| tls.mode) {
        None => UpstreamRequestMode::PlainHttp,
        Some(UpstreamTlsMode::Simple) => UpstreamRequestMode::SimpleTls,
        Some(UpstreamTlsMode::DubboMutual) => UpstreamRequestMode::DubboMutual,
    }
}

pub(super) fn host_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|host| host.split(':').next().unwrap_or(host))
}

pub(super) fn header_pairs(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect()
}
