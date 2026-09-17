use crate::ProxyState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{
    HeaderMap, HeaderValue as HttpHeaderValue, Request, Response, StatusCode, Uri, Version,
};
use axum::routing::any;
use axum::Router;
use hyper::body::Bytes;
use std::env;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn, Instrument};
use transit::{
    AgentProtocol, AgentRoute, Backend, ConfigSnapshot, MatchInput, RetryPolicy,
    WeightedBackend, HTTP_LISTENER_PORT,
};

mod context;
mod detect;
mod headers;
mod listeners;
mod routing;
mod upstream;

use context::{apply_stream_headers, AgentRequestContext};
use detect::{detect_agent_protocol, is_grpc_request};
use headers::{
    apply_provider_headers, remove_connection_headers,
};
use routing::{
    backend_matches_protocol, backend_provider, compose_backend_uri, endpoint_authority,
    header_pairs, host_header, protocol_name, upstream_request_mode, UpstreamRequestMode,
};
use upstream::UpstreamClients;

const DEFAULT_MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone)]
pub struct ProxyServer {
    state: ProxyState,
    clients: UpstreamClients,
    metrics_identity: MetricsIdentity,
    max_body_bytes: usize,
    listener_port: u16,
}

impl ProxyServer {
    pub fn new(state: ProxyState) -> Self {
        Self {
            state,
            clients: UpstreamClients::from_env(),
            metrics_identity: MetricsIdentity::from_env(),
            max_body_bytes: parse_max_body_bytes(
                env::var("TRANSIT_MAX_BODY_BYTES").ok().as_deref(),
            ),
            listener_port: HTTP_LISTENER_PORT,
        }
    }

    pub async fn serve(self, addr: SocketAddr) -> std::io::Result<()> {
        self.serve_with_shutdown(addr, std::future::pending::<()>())
            .await
    }

    /// Serves until `shutdown` resolves, then stops accepting and lets in-flight
    /// requests finish before returning.
    pub async fn serve_with_shutdown(
        self,
        addr: SocketAddr,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> std::io::Result<()> {
        let mut server = self;
        server.listener_port = addr.port();
        let tls = server.state.access_settings().active_tls();
        let app = Router::new()
            .fallback(any(proxy_request))
            .with_state(server);
        crate::access_settings::serve_router(app, addr, tls, shutdown).await
    }
}

#[derive(Clone)]
struct MetricsIdentity {
    namespace: String,
    gateway: String,
}

impl MetricsIdentity {
    fn from_env() -> Self {
        Self {
            namespace: env::var("POD_NAMESPACE").unwrap_or_else(|_| "unknown".to_string()),
            gateway: env::var("TRANSIT_GATEWAY_NAME")
                .or_else(|_| env::var("GATEWAY_NAME"))
                .unwrap_or_else(|_| "unknown".to_string()),
        }
    }
}

fn parse_max_body_bytes(value: Option<&str>) -> usize {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(DEFAULT_MAX_BODY_BYTES)
}

// Buffered reads back agent-route retries and body inspection; the limit keeps a
// single oversized request from exhausting proxy memory.
async fn read_body_limited(
    headers: &HeaderMap,
    mut body: Body,
    limit: usize,
) -> Result<Bytes, (StatusCode, String)> {
    use hyper::body::HttpBody;

    if let Some(length) = headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
    {
        if length > limit {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("request body of {length} bytes exceeds limit of {limit} bytes"),
            ));
        }
    }

    let mut buf = Vec::new();
    while let Some(chunk) = body.data().await {
        let chunk =
            chunk.map_err(|e| (StatusCode::BAD_REQUEST, format!("read request body: {e}")))?;
        if buf.len() + chunk.len() > limit {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("request body exceeds limit of {limit} bytes"),
            ));
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(Bytes::from(buf))
}

#[derive(Clone)]
struct GatewayCredentialHeaders(Vec<http::HeaderName>);

async fn proxy_request(State(server): State<ProxyServer>, req: Request<Body>) -> Response<Body> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let span = tracing::info_span!(
        "transit.request",
        http.method = %method,
        http.target = %path,
        http.status_code = tracing::field::Empty,
        http.latency_ms = tracing::field::Empty,
        gateway.namespace = tracing::field::Empty,
        gateway.name = tracing::field::Empty,
        http.route = tracing::field::Empty,
        transit.cluster = tracing::field::Empty,
        upstream.address = tracing::field::Empty
    );
    // Held for the whole handler
    let _in_flight = server.state.track_request();
    let result = forward(server, req).instrument(span.clone()).await;
    match result {
        Ok(resp) => {
            span.record("http.status_code", resp.status().as_u16());
            resp
        }
        Err((status, message)) => {
            span.record("http.status_code", status.as_u16());
            warn!(status = status.as_u16(), %message, "request failed");
            Response::builder()
                .status(status)
                .body(Body::from(message))
                .unwrap_or_else(|_| Response::new(Body::from("proxy error")))
        }
    }
}

async fn forward(
    server: ProxyServer,
    mut req: Request<Body>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let (mut parts, body) = req.into_parts();
    let mut upstream_headers = parts.headers.clone();
    if !server
        .state
        .access_settings()
        .authenticate_api(&mut upstream_headers, &mut parts.uri)
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid or missing gateway API key".into(),
        ));
    }
    let consumed = parts
        .headers
        .keys()
        .filter(|name| !upstream_headers.contains_key(*name))
        .cloned()
        .collect();
    parts.extensions.insert(GatewayCredentialHeaders(consumed));
    req = Request::from_parts(parts, body);
    let snapshot = server.state.snapshot();

    // gRPC and Dubbo Triple require end-to-end HTTP/2 with streaming bodies and
    // trailer propagation; buffering the body here would break both.
    if is_grpc_request(req.headers()) {
        return forward_http(server, snapshot, req).await;
    }
    if detect_agent_protocol(req.uri().path()).is_some() || snapshot.has_agent_routes() {
        let (parts, body) = req.into_parts();
        let body_bytes = read_body_limited(&parts.headers, body, server.max_body_bytes).await?;
        let candidates: &[AgentProtocol] = &[AgentProtocol::Http];
        for protocol in candidates {
            let context = AgentRequestContext::new(*protocol, &parts, &body_bytes);
            if let Some(route) = snapshot
                .agent_route_for_port(server.listener_port, &context.input())
                .cloned()
            {
                return forward_agent(server.clone(), snapshot, parts, body_bytes, context, route)
                    .await;
            }
        }
        req = Request::from_parts(parts, Body::from(body_bytes));
    }

    forward_http(server, snapshot, req).await
}

async fn forward_http(
    server: ProxyServer,
    snapshot: Arc<ConfigSnapshot>,
    mut req: Request<Body>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let method = req.method().as_str().to_string();
    let host = host_header(req.headers()).unwrap_or("*").to_string();
    let path = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();
    let headers = header_pairs(req.headers());
    let input = MatchInput {
        host: &host,
        path: &path,
        headers: &headers,
    };

    let route = match snapshot
        .route_for(server.listener_port, &input)
        .or_else(|_| snapshot.route_for_unique_port(&input))
    {
        Ok(route) => route,
        Err(err) => {
            record_http_observation(
                &server,
                HttpObservation {
                    route: "none",
                    cluster: "none",
                    method: &method,
                    host: &host,
                    path: &path,
                    status_code: StatusCode::NOT_FOUND.as_u16(),
                    latency_ms: 0,
                    upstream: "none",
                },
            );
            return Err((StatusCode::NOT_FOUND, err.to_string()));
        }
    };
    let route_name = route.name.clone();
    record_http_span(&server, &route_name, "none", "none", 0, 0);
    let weighted_clusters = route.weighted_clusters.clone();

    let weighted = match server
        .state
        .pick_cluster(&route_name, &weighted_clusters)
        .await
    {
        Some(weighted) => weighted,
        None => {
            record_http_observation(
                &server,
                HttpObservation {
                    route: &route_name,
                    cluster: "none",
                    method: &method,
                    host: &host,
                    path: &path,
                    status_code: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                    latency_ms: 0,
                    upstream: "none",
                },
            );
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "route has no clusters".to_string(),
            ));
        }
    };

    let cluster = match snapshot.cluster(&weighted.name) {
        Some(cluster) => cluster.clone(),
        None => {
            record_http_observation(
                &server,
                HttpObservation {
                    route: &route_name,
                    cluster: &weighted.name,
                    method: &method,
                    host: &host,
                    path: &path,
                    status_code: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                    latency_ms: 0,
                    upstream: "none",
                },
            );
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("cluster {} not found", weighted.name),
            ));
        }
    };
    let cluster_name = cluster.name.clone();

    let endpoint = match server.state.pick_endpoint(&cluster).await {
        Ok(endpoint) => endpoint,
        Err(err) => {
            record_http_observation(
                &server,
                HttpObservation {
                    route: &route_name,
                    cluster: &cluster_name,
                    method: &method,
                    host: &host,
                    path: &path,
                    status_code: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                    latency_ms: 0,
                    upstream: "none",
                },
            );
            return Err((StatusCode::SERVICE_UNAVAILABLE, err.to_string()));
        }
    };
    let upstream = endpoint_authority(endpoint);
    record_http_span(&server, &route_name, &cluster_name, &upstream, 0, 0);
    let _circuit_breaker_permit = match server.state.try_acquire_circuit_breaker(&cluster) {
        Ok(permit) => permit,
        Err(_) => {
            record_http_observation(
                &server,
                HttpObservation {
                    route: &route_name,
                    cluster: &cluster_name,
                    method: &method,
                    host: &host,
                    path: &path,
                    status_code: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                    latency_ms: 0,
                    upstream: &upstream,
                },
            );
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                format!("cluster {} circuit breaker open", cluster_name),
            ));
        }
    };

    let tls = cluster.tls.as_ref();
    let request_mode = upstream_request_mode(tls);
    let scheme = match request_mode {
        UpstreamRequestMode::PlainHttp => "http",
        UpstreamRequestMode::SimpleTls | UpstreamRequestMode::DubboMutual => "https",
    };
    let upstream_uri = format!("{}://{}{}", scheme, upstream, path)
        .parse::<Uri>()
        .map_err(|e| {
            record_http_observation(
                &server,
                HttpObservation {
                    route: &route_name,
                    cluster: &cluster_name,
                    method: &method,
                    host: &host,
                    path: &path,
                    status_code: StatusCode::BAD_GATEWAY.as_u16(),
                    latency_ms: 0,
                    upstream: &upstream,
                },
            );
            (
                StatusCode::BAD_GATEWAY,
                format!("invalid upstream uri: {e}"),
            )
        })?;

    debug!(
        route = %route_name,
        cluster = %cluster_name,
        endpoint = %endpoint.address,
        upstream_mode = ?request_mode,
        "forwarding request"
    );

    let use_h2 = cluster.http2 || is_grpc_request(req.headers());
    if let Some(consumed) = req.extensions().get::<GatewayCredentialHeaders>().cloned() {
        for name in consumed.0 {
            req.headers_mut().remove(name);
        }
    }
    *req.uri_mut() = upstream_uri;
    req.headers_mut().remove(http::header::HOST);
    *req.version_mut() = if use_h2 {
        remove_connection_headers(req.headers_mut());
        Version::HTTP_2
    } else {
        Version::HTTP_11
    };

    let _route_in_flight = server.state.track_route_request(
        &server.metrics_identity.namespace,
        &server.metrics_identity.gateway,
        &route_name,
        &cluster_name,
    );
    let started = Instant::now();
    let result = match (request_mode, use_h2) {
        (UpstreamRequestMode::PlainHttp, false) => server.clients.request_plain(req).await,
        (UpstreamRequestMode::PlainHttp, true) | (UpstreamRequestMode::SimpleTls, true) => {
            server.clients.request_h2(req).await
        }
        (UpstreamRequestMode::SimpleTls, false) => server.clients.request_web(req).await,
        (UpstreamRequestMode::DubboMutual, h2) => {
            let tls = tls.expect("dubbo mutual request mode requires TLS config");
            server
                .clients
                .request_mtls(&cluster, tls, &snapshot, req, h2)
                .await
        }
    };
    let latency_ms = started.elapsed().as_millis() as u64;
    let status = result
        .as_ref()
        .map(|response| response.status().as_u16())
        .unwrap_or_else(|(status, _)| status.as_u16());
    server
        .state
        .record_endpoint_result(&cluster, endpoint, status);
    record_http_observation(
        &server,
        HttpObservation {
            route: &route_name,
            cluster: &cluster_name,
            method: &method,
            host: &host,
            path: &path,
            status_code: status,
            latency_ms,
            upstream: &upstream,
        },
    );
    result
}

#[allow(dead_code)]
struct HttpObservation<'a> {
    route: &'a str,
    cluster: &'a str,
    method: &'a str,
    host: &'a str,
    path: &'a str,
    status_code: u16,
    latency_ms: u64,
    upstream: &'a str,
}

fn record_http_observation(server: &ProxyServer, observation: HttpObservation<'_>) {
    record_http_metric(
        server,
        observation.route,
        observation.cluster,
        observation.method,
        observation.status_code,
        observation.latency_ms,
    );
    record_http_span(
        server,
        observation.route,
        observation.cluster,
        observation.upstream,
        observation.status_code,
        observation.latency_ms,
    );
}

fn record_http_span(
    server: &ProxyServer,
    route: &str,
    cluster: &str,
    upstream: &str,
    status_code: u16,
    latency_ms: u64,
) {
    let span = tracing::Span::current();
    span.record(
        "gateway.namespace",
        server.metrics_identity.namespace.as_str(),
    );
    span.record("gateway.name", server.metrics_identity.gateway.as_str());
    span.record("http.route", route);
    span.record("transit.cluster", cluster);
    span.record("upstream.address", upstream);
    if status_code > 0 {
        span.record("http.status_code", status_code);
    }
    span.record("http.latency_ms", latency_ms);
}

fn record_http_metric(
    server: &ProxyServer,
    route: &str,
    cluster: &str,
    method: &str,
    status_code: u16,
    latency_ms: u64,
) {
    server.state.record_http_request(
        &server.metrics_identity.namespace,
        &server.metrics_identity.gateway,
        route,
        cluster,
        method,
        status_code,
        latency_ms,
    );
}

async fn forward_agent(
    server: ProxyServer,
    snapshot: Arc<ConfigSnapshot>,
    parts: http::request::Parts,
    body: Bytes,
    mut context: AgentRequestContext,
    route: Arc<AgentRoute>,
) -> Result<Response<Body>, (StatusCode, String)> {
    apply_agent_path_rewrite(&route, &mut context);

    let eligible = route
        .weighted_backends
        .iter()
        .filter_map(|weighted| {
            let backend = snapshot.backend(&weighted.name)?;
            if backend_matches_protocol(backend, context.protocol) {
                Some(weighted.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if eligible.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("agent route {} has no eligible backends", route.name),
        ));
    }

    let primary = server
        .state
        .pick_backend(&route.name, &eligible)
        .await
        .cloned()
        .unwrap_or_else(|| eligible[0].clone());
    let mut ordered = vec![primary.clone()];
    ordered.extend(eligible.iter().filter(|b| b.name != primary.name).cloned());

    request_agent_with_failover(
        &server,
        &snapshot,
        &route,
        &ordered,
        &parts,
        &body,
        &context,
    )
    .await
}

fn apply_agent_path_rewrite(route: &AgentRoute, context: &mut AgentRequestContext) {
    let Some(replacement) = route.replace_prefix_match.as_deref() else {
        return;
    };
    let input = context.input();
    let Some(path_match) = route
        .matches
        .iter()
        .find(|candidate| candidate.matches(&input))
        .map(|candidate| &candidate.path)
    else {
        return;
    };
    let suffix = match path_match {
        transit::PathMatch::Prefix(prefix) => context.path.strip_prefix(prefix),
        transit::PathMatch::Exact(exact) if context.path == *exact => Some(""),
        _ => None,
    };
    let Some(suffix) = suffix else {
        return;
    };
    let query = context
        .path_and_query
        .strip_prefix(&context.path)
        .unwrap_or_default();
    context.path = format!("{replacement}{suffix}");
    context.path_and_query = format!("{}{query}", context.path);
}

async fn request_agent_with_failover(
    server: &ProxyServer,
    snapshot: &ConfigSnapshot,
    route: &AgentRoute,
    ordered: &[WeightedBackend],
    parts: &http::request::Parts,
    body: &Bytes,
    context: &AgentRequestContext,
) -> Result<Response<Body>, (StatusCode, String)> {
    let retry = RetryPolicy {
        attempts: 1,
        statuses: vec![502, 503, 504],
    };
    let attempts = retry.attempts.max(1) as usize;
    let mut last_error = None;

    for attempt in 0..attempts {
        for weighted in ordered {
            let Some(backend) = snapshot.backend(&weighted.name) else {
                continue;
            };
            let started = Instant::now();
            let upstream_span = tracing::info_span!(
                "transit.agent.upstream",
                protocol = protocol_name(context.protocol),
                route = %route.name,
                backend = %backend.name,
                http.status_code = tracing::field::Empty
            );
            match request_agent_backend(
                server,
                snapshot,
                route,
                backend,
                parts,
                body,
                context,
            )
            .instrument(upstream_span.clone())
            .await
            {
                Ok(mut response) => {
                    let status = response.status();
                    upstream_span.record("http.status_code", status.as_u16());
                    let req_latency_ms = started.elapsed().as_millis() as u64;
                    server.state.record_agent_request(
                        protocol_name(context.protocol),
                        &route.name,
                        &backend.name,
                        status.as_u16(),
                        req_latency_ms,
                    );
                    apply_stream_headers(response.headers_mut(), context);
                    if attempt + 1 < attempts && retry.statuses.contains(&status.as_u16()) {
                        last_error = Some((status, "retrying after status".to_string()));
                        continue;
                    }
                    return Ok(response);
                }
                Err(err) => {
                    upstream_span.record("http.status_code", err.0.as_u16());
                    let req_latency_ms = started.elapsed().as_millis() as u64;
                    server.state.record_agent_request(
                        protocol_name(context.protocol),
                        &route.name,
                        &backend.name,
                        err.0.as_u16(),
                        req_latency_ms,
                    );
                    last_error = Some(err);
                }
            }
        }
    }

    Err(last_error.unwrap_or((
        StatusCode::SERVICE_UNAVAILABLE,
        format!("all backends for route {} failed", route.name),
    )))
}

async fn request_agent_backend(
    server: &ProxyServer,
    snapshot: &ConfigSnapshot,
    _route: &AgentRoute,
    backend: &Backend,
    parts: &http::request::Parts,
    body: &Bytes,
    context: &AgentRequestContext,
) -> Result<Response<Body>, (StatusCode, String)> {
    let provider = backend_provider(snapshot, backend);
    let endpoint = backend.endpoint(provider).ok_or_else(|| {
        (
            StatusCode::BAD_GATEWAY,
            format!("backend {} has no endpoint", backend.name),
        )
    })?;

    let uri = compose_backend_uri(context.protocol, endpoint, &context.path_and_query)?;
    let mut headers = parts.headers.clone();
    if let Some(consumed) = parts.extensions.get::<GatewayCredentialHeaders>() {
        for name in &consumed.0 {
            headers.remove(name);
        }
    }
    let provider_secret = provider
        .and_then(|value| value.credential_ref.as_ref())
        .and_then(|reference| server.state.credential(reference));
    apply_provider_headers(&mut headers, provider, provider_secret.as_deref());
    headers.remove(http::header::HOST);
    headers.remove(http::header::TRANSFER_ENCODING);
    if !body.is_empty() || headers.contains_key(http::header::CONTENT_LENGTH) {
        headers.insert(
            http::header::CONTENT_LENGTH,
            HttpHeaderValue::from(body.len()),
        );
    }

    let mut builder = Request::builder()
        .method(parts.method.clone())
        .uri(uri)
        .version(Version::HTTP_11);
    *builder.headers_mut().unwrap() = headers;
    let request = builder.body(Body::from(body.clone())).map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("build upstream request: {e}"),
        )
    })?;

    server.clients.request_web(request).await
}

#[cfg(test)]
mod tests {
    use super::upstream::GrpcBootstrap;
    use super::*;
    use std::path::Path;

    #[test]
    fn max_body_bytes_parses_env_values() {
        assert_eq!(parse_max_body_bytes(None), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(parse_max_body_bytes(Some("")), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(parse_max_body_bytes(Some("0")), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(parse_max_body_bytes(Some("abc")), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(parse_max_body_bytes(Some(" 4096 ")), 4096);
    }

    #[tokio::test]
    async fn read_body_limited_rejects_oversized_content_length() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_LENGTH,
            HttpHeaderValue::from_static("32"),
        );

        let (status, _) = read_body_limited(&headers, Body::from("ignored"), 16)
            .await
            .unwrap_err();
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn read_body_limited_rejects_oversized_stream_without_content_length() {
        let (mut sender, body) = Body::channel();
        let writer = tokio::spawn(async move {
            for _ in 0..4 {
                if sender.send_data(Bytes::from(vec![0u8; 8])).await.is_err() {
                    return;
                }
            }
        });

        let (status, _) = read_body_limited(&HeaderMap::new(), body, 16)
            .await
            .unwrap_err();
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn read_body_limited_passes_body_within_limit() {
        let bytes = read_body_limited(&HeaderMap::new(), Body::from("hello"), 16)
            .await
            .unwrap();
        assert_eq!(&bytes[..], b"hello");
    }

    #[test]
    fn parses_grpc_xds_bootstrap_file_watcher_provider() {
        let bootstrap = serde_json::from_str::<GrpcBootstrap>(
            r#"{
              "certificate_providers": {
                "default": {
                  "plugin_name": "file_watcher",
                  "config": {
                    "certificate_file": "/etc/dubbo/proxy/cert-chain.pem",
                    "private_key_file": "/etc/dubbo/proxy/key.pem",
                    "ca_certificate_file": "/etc/dubbo/proxy/root-cert.pem"
                  }
                }
              }
            }"#,
        )
        .unwrap();

        let provider = bootstrap.provider("default").unwrap();
        assert_eq!(
            provider
                .required_path("certificate_file", "default")
                .unwrap(),
            Path::new("/etc/dubbo/proxy/cert-chain.pem")
        );
        assert_eq!(
            provider
                .required_path("private_key_file", "default")
                .unwrap(),
            Path::new("/etc/dubbo/proxy/key.pem")
        );
        assert_eq!(
            provider
                .required_path("ca_certificate_file", "default")
                .unwrap(),
            Path::new("/etc/dubbo/proxy/root-cert.pem")
        );
    }
}
