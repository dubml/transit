use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode, Uri, Version};
use axum::routing::any;
use axum::Router;
use std::env;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, warn, Instrument};
use crate::{MatchInput, RuntimeConfig, HTTP_LISTENER_PORT};

mod detect;
mod headers;
mod listeners;
mod routing;
mod upstream;

use detect::is_grpc_request;
use headers::remove_connection_headers;
use routing::{header_pairs, host_header};
use upstream::UpstreamClients;

#[derive(Clone)]
pub struct ProxyServer {
    config: Arc<RuntimeConfig>,
    llm_accounts: Arc<crate::llm::LlmAccounts>,
    access_settings: Arc<crate::llm::access_settings::AccessSettings>,
    clients: UpstreamClients,
    #[allow(dead_code)]
    metrics_identity: MetricsIdentity,
    listener_port: u16,
}

impl ProxyServer {
    pub fn new(config: Arc<RuntimeConfig>) -> Self {
        Self {
            config,
            llm_accounts: Arc::new(crate::llm::LlmAccounts::default()),
            access_settings: Arc::new(crate::llm::access_settings::AccessSettings::default()),
            clients: UpstreamClients::from_env(),
            metrics_identity: MetricsIdentity::from_env(),
            listener_port: HTTP_LISTENER_PORT,
        }
    }

    pub fn config(&self) -> &Arc<RuntimeConfig> {
        &self.config
    }

    pub fn llm_accounts(&self) -> &crate::llm::LlmAccounts {
        &self.llm_accounts
    }

    pub fn access_settings(&self) -> &crate::llm::access_settings::AccessSettings {
        &self.access_settings
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
        let tls = server.access_settings.active_tls();
        let app = Router::new()
            .fallback(any(proxy_request))
            .with_state(server);
        crate::llm::access_settings::serve_router(app, addr, tls, shutdown).await
    }
}

#[derive(Clone)]
#[allow(dead_code)]
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
        .access_settings
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

    forward_http(server, req).await
}

async fn forward_http(
    server: ProxyServer,
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
        method: Some(&method),
    };

    let route = match server.config.route_for(server.listener_port, &input) {
        Ok(route) => route,
        Err(err) => {
            return Err((StatusCode::NOT_FOUND, err.to_string()));
        }
    };
    let route_name = route.name.clone();

    let target_hosts = route.target_hosts();
    let endpoint_raw = match target_hosts.first() {
        Some(ep) => (*ep).to_string(),
        None => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "route has no endpoints or backends".to_string(),
            ));
        }
    };

    let is_https = endpoint_raw.starts_with("https://");
    let clean_endpoint = endpoint_raw
        .strip_prefix("http://")
        .or_else(|| endpoint_raw.strip_prefix("https://"))
        .unwrap_or(&endpoint_raw);
    let scheme = if is_https { "https" } else { "http" };

    let upstream = clean_endpoint.to_string();

    let upstream_uri = format!("{}://{}{}", scheme, upstream, path)
        .parse::<Uri>()
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("invalid upstream uri: {e}"),
            )
        })?;

    debug!(
        route = %route_name,
        endpoint = %upstream,
        "forwarding request"
    );

    let use_h2 = is_grpc_request(req.headers());
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

    let result = if is_https {
        server.clients.request_web(req).await
    } else if use_h2 {
        server.clients.request_h2(req).await
    } else {
        server.clients.request_plain(req).await
    };

    let mut response = result?;
    remove_connection_headers(response.headers_mut());
    Ok(response)
}
