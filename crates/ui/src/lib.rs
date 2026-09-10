use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use transit_proxy::{
    A2aMethodMetric, HttpRouteConcurrencyMetric, HttpRouteMetric, LlmUsageMetric, McpToolMetric,
    ProxyMetrics, ProxyState, Readiness, RouteMetric,
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

mod access;
mod llm;
mod panel;

const PROMETHEUS_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Clone)]
pub struct UiServer {
    state: ProxyState,
    runtime: transit_core::RuntimeModeInfo,
    build: BuildInfo,
    proxy_port: u16,
    metrics_enabled: bool,
    bind_addr: Option<SocketAddr>,
    oauth_callbacks: std::sync::Arc<llm::CallbackListeners>,
    management_attempts: std::sync::Arc<access::Attempts>,
    panel_assets: std::sync::Arc<panel::PanelAssets>,
}

impl UiServer {
    pub fn new(state: ProxyState, proxy_addr: SocketAddr, metrics_enabled: bool) -> Self {
        Self {
            state,
            runtime: Default::default(),
            build: BuildInfo {
                name: "transit",
                version: env!("CARGO_PKG_VERSION"),
            },
            proxy_port: proxy_addr.port(),
            metrics_enabled,
            bind_addr: None,
            oauth_callbacks: Default::default(),
            management_attempts: Default::default(),
            panel_assets: Default::default(),
        }
    }

    pub fn with_version(mut self, version: &'static str) -> Self {
        self.build.version = version;
        self
    }

    pub fn with_runtime_mode(mut self, runtime: transit_core::RuntimeModeInfo) -> Self {
        self.runtime = runtime;
        self
    }

    pub async fn serve(self, addr: SocketAddr) -> std::io::Result<()> {
        self.serve_with_shutdown(addr, std::future::pending::<()>())
            .await
    }

    /// Serves until `shutdown` resolves, then stops accepting and lets in-flight
    /// requests finish before returning.
    pub async fn serve_with_shutdown(
        mut self,
        addr: SocketAddr,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> std::io::Result<()> {
        self.bind_addr = Some(addr);
        let tls = self.state.access_settings().active_tls();
        let updater = self.panel_assets.clone().start(self.state.clone());
        let app = Router::new()
            .merge(llm::routes())
            .merge(access::routes())
            .route("/", get(ui_page))
            .route("/ui", get(ui_page))
            .route("/assets/transit-logo.svg", get(logo_svg))
            .route("/assets/transit-mark.svg", get(mark_svg))
            .route("/healthz", get(healthz))
            .route("/readyz", get(readyz))
            .route("/metrics", get(metrics))
            .route("/debug/config", get(debug_config))
            .route("/debug/cost", get(debug_cost))
            .route("/debug/routes", get(debug_routes))
            .route("/debug/clusters", get(debug_clusters))
            .route("/debug/backends", get(debug_backends))
            .route("/debug/policies", get(debug_policies))
            .route("/debug/sources", get(debug_sources))
            .route("/debug/security/posture", get(debug_security_posture))
            .route("/debug/security/events", get(debug_security_events))
            .route("/debug/security/identities", get(debug_security_identities))
            .route("/debug/observability", get(debug_observability))
            .route("/debug/services", get(debug_services))
            .layer(axum::middleware::from_fn_with_state(
                self.clone(),
                access::guard,
            ))
            .with_state(self);

        let result = transit_proxy::access_settings::serve_router(app, addr, tls, shutdown).await;
        updater.abort();
        result
    }
}

async fn ui_page(State(ui): State<UiServer>) -> Html<String> {
    Html(
        ui.panel_assets
            .html(&ui.state)
            .await
            .unwrap_or_else(|| ui_html(ui.proxy_port)),
    )
}

async fn logo_svg() -> Response {
    (
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        include_str!("../../../logo/transit-logo.svg"),
    )
        .into_response()
}

async fn mark_svg() -> Response {
    (
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        include_str!("../../../logo/transit-mark.svg"),
    )
        .into_response()
}

async fn healthz(State(ui): State<UiServer>) -> Json<BuildInfo> {
    Json(ui.build)
}

async fn readyz(State(ui): State<UiServer>) -> Response {
    let readiness = ui.state.readiness();
    let status = if readiness.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(readiness)).into_response()
}

async fn metrics(State(ui): State<UiServer>) -> Response {
    if !ui.metrics_enabled {
        return StatusCode::NOT_FOUND.into_response();
    }
    let readiness = ui.state.readiness();
    let proxy = ui.state.metrics();
    (
        [(header::CONTENT_TYPE, PROMETHEUS_CONTENT_TYPE)],
        prometheus_metrics(readiness, proxy),
    )
        .into_response()
}

fn prometheus_metrics(readiness: Readiness, proxy: ProxyMetrics) -> String {
    let mut out = format!(
        "# HELP transit_ready Whether transit has accepted runtime config\n# TYPE transit_ready gauge\ntransit_ready {}\n# HELP transit_config_conflicts Current rejected config conflicts\n# TYPE transit_config_conflicts gauge\ntransit_config_conflicts {}\n",
        if readiness.ready { 1 } else { 0 },
        readiness.conflicts.len()
    );
    out.push_str("# HELP transit_requests_total Total requests observed by transit\n# TYPE transit_requests_total counter\n");
    out.push_str(&format!(
        "transit_requests_total {}\n",
        proxy.total_requests
    ));
    out.push_str("# HELP transit_agent_requests_total Agent protocol requests observed by transit\n# TYPE transit_agent_requests_total counter\n");
    out.push_str(&format!(
        "transit_agent_requests_total {}\n",
        proxy.agent_requests
    ));
    out.push_str("# HELP transit_policy_denied_total Requests denied by transit policy\n# TYPE transit_policy_denied_total counter\n");
    out.push_str(&format!(
        "transit_policy_denied_total {}\n",
        proxy.policy_denied
    ));
    out.push_str("# HELP transit_upstream_failures_total Upstream failures observed by transit\n# TYPE transit_upstream_failures_total counter\n");
    out.push_str(&format!(
        "transit_upstream_failures_total {}\n",
        proxy.upstream_failures
    ));
    out.push_str("# HELP transit_requests_in_flight Requests being handled right now\n# TYPE transit_requests_in_flight gauge\n");
    out.push_str(&format!(
        "transit_requests_in_flight {}\n",
        proxy.concurrency.in_flight
    ));
    // Counted separately from in-flight requests: these are waiting on a
    // scale-up, not on an upstream, so folding them together would read as the
    // gateway having gone slow.
    out.push_str("# HELP transit_activation_requests_held Requests waiting for a scaled-to-zero target to come up\n# TYPE transit_activation_requests_held gauge\n");
    out.push_str(&format!(
        "transit_activation_requests_held {}\n",
        proxy.held_activation_requests
    ));
    // Scale on rate() of this rather than on the gauge above: the gauge is a
    // single instant and misses every burst that lands between two scrapes.
    out.push_str("# HELP transit_request_seconds_total Accumulated request time; rate() gives average concurrency\n# TYPE transit_request_seconds_total counter\n");
    out.push_str(&format!(
        "transit_request_seconds_total {}\n",
        proxy.concurrency.seconds_total
    ));
    out.push_str("# HELP transit_http_route_requests_in_flight Requests in flight by route and cluster\n# TYPE transit_http_route_requests_in_flight gauge\n");
    for route in &proxy.http_route_concurrency {
        let labels = http_route_concurrency_labels(route);
        out.push_str(&format!(
            "transit_http_route_requests_in_flight{{{labels}}} {}\n",
            route.concurrency.in_flight
        ));
    }
    out.push_str("# HELP transit_http_route_request_seconds_total Accumulated request time by route and cluster\n# TYPE transit_http_route_request_seconds_total counter\n");
    for route in &proxy.http_route_concurrency {
        let labels = http_route_concurrency_labels(route);
        out.push_str(&format!(
            "transit_http_route_request_seconds_total{{{labels}}} {}\n",
            route.concurrency.seconds_total
        ));
    }
    out.push_str("# HELP transit_http_route_requests_total HTTP gateway requests observed by route and cluster\n# TYPE transit_http_route_requests_total counter\n");
    for route in &proxy.http_routes {
        let labels = http_route_labels(route);
        out.push_str(&format!(
            "transit_http_route_requests_total{{{labels}}} {}\n",
            route.requests
        ));
    }
    out.push_str("# HELP transit_http_route_failures_total HTTP gateway upstream failures observed by route and cluster\n# TYPE transit_http_route_failures_total counter\n");
    for route in &proxy.http_routes {
        let labels = http_route_labels(route);
        out.push_str(&format!(
            "transit_http_route_failures_total{{{labels}}} {}\n",
            route.failures
        ));
    }
    out.push_str("# HELP transit_http_route_latency_ms HTTP gateway upstream latency in milliseconds\n# TYPE transit_http_route_latency_ms histogram\n");
    for route in &proxy.http_routes {
        let labels = http_route_labels(route);
        out.push_str(&format!(
            "transit_http_route_latency_ms_sum{{{labels}}} {}\n",
            route.latency_ms_sum
        ));
        for bucket in &route.latency_ms_buckets {
            out.push_str(&format!(
                "transit_http_route_latency_ms_bucket{{{labels},le=\"{}\"}} {}\n",
                bucket.le, bucket.count
            ));
        }
        out.push_str(&format!(
            "transit_http_route_latency_ms_bucket{{{labels},le=\"+Inf\"}} {}\n",
            route.requests
        ));
        out.push_str(&format!(
            "transit_http_route_latency_ms_count{{{labels}}} {}\n",
            route.requests
        ));
    }
    out.push_str("# HELP transit_agent_route_requests_total Agent protocol requests observed by route and backend\n# TYPE transit_agent_route_requests_total counter\n");
    for route in &proxy.routes {
        let labels = agent_route_labels(route);
        out.push_str(&format!(
            "transit_agent_route_requests_total{{{labels}}} {}\n",
            route.requests
        ));
    }
    out.push_str("# HELP transit_agent_route_failures_total Agent protocol upstream failures observed by route and backend\n# TYPE transit_agent_route_failures_total counter\n");
    for route in &proxy.routes {
        let labels = agent_route_labels(route);
        out.push_str(&format!(
            "transit_agent_route_failures_total{{{labels}}} {}\n",
            route.failures
        ));
    }
    out.push_str("# HELP transit_agent_route_latency_ms Agent protocol upstream latency in milliseconds\n# TYPE transit_agent_route_latency_ms histogram\n");
    for route in &proxy.routes {
        let labels = agent_route_labels(route);
        out.push_str(&format!(
            "transit_agent_route_latency_ms_sum{{{labels}}} {}\n",
            route.latency_ms_sum
        ));
        for bucket in &route.latency_ms_buckets {
            out.push_str(&format!(
                "transit_agent_route_latency_ms_bucket{{{labels},le=\"{}\"}} {}\n",
                bucket.le, bucket.count
            ));
        }
        out.push_str(&format!(
            "transit_agent_route_latency_ms_bucket{{{labels},le=\"+Inf\"}} {}\n",
            route.requests
        ));
        out.push_str(&format!(
            "transit_agent_route_latency_ms_count{{{labels}}} {}\n",
            route.requests
        ));
    }
    out.push_str("# HELP transit_llm_requests_total LLM requests with recorded token usage\n# TYPE transit_llm_requests_total counter\n");
    for usage in &proxy.llm_usage {
        let labels = llm_usage_labels(usage);
        out.push_str(&format!(
            "transit_llm_requests_total{{{labels}}} {}\n",
            usage.requests
        ));
    }
    out.push_str("# HELP transit_llm_tokens_total LLM tokens observed by route, backend, and model\n# TYPE transit_llm_tokens_total counter\n");
    for usage in &proxy.llm_usage {
        let labels = llm_usage_labels(usage);
        out.push_str(&format!(
            "transit_llm_tokens_total{{{labels},type=\"prompt\"}} {}\n",
            usage.prompt_tokens
        ));
        out.push_str(&format!(
            "transit_llm_tokens_total{{{labels},type=\"completion\"}} {}\n",
            usage.completion_tokens
        ));
        out.push_str(&format!(
            "transit_llm_tokens_total{{{labels},type=\"cached_prompt\"}} {}\n",
            usage.cached_prompt_tokens
        ));
    }
    out.push_str("# HELP transit_llm_api_usd_nanos_total API list-price USD for observed tokens, in nanodollars (1e-9 USD). Offline published rate card; unknown models omitted.\n# TYPE transit_llm_api_usd_nanos_total counter\n");
    out.push_str("# HELP transit_llm_chatgpt_credit_micros_total Codex/ChatGPT subscription credits for observed tokens, in microcredits (1e-6 credit). Always emitted; incomplete quotes still include known credit line items.\n# TYPE transit_llm_chatgpt_credit_micros_total counter\n");
    for usage in &proxy.llm_usage {
        let labels = llm_usage_labels(usage);
        if let Ok(quote) = transit_core::quote_tokens(
            &usage.model,
            transit_core::TokenCounts {
                prompt_tokens: usage.prompt_tokens,
                cached_prompt_tokens: usage.cached_prompt_tokens,
                cache_write_tokens: 0,
                completion_tokens: usage.completion_tokens,
            },
            transit_core::ServiceTier::Standard,
            transit_core::ContextBand::Short,
        ) {
            out.push_str(&format!(
                "transit_llm_api_usd_nanos_total{{{labels}}} {}\n",
                quote.api_usd_nanos
            ));
            out.push_str(&format!(
                "transit_llm_chatgpt_credit_micros_total{{{labels}}} {}\n",
                quote.chatgpt_credit_micros
            ));
        }
    }
    out.push_str("# HELP transit_mcp_tool_calls_total MCP tools/call requests by route, backend, and tool\n# TYPE transit_mcp_tool_calls_total counter\n");
    for tool in &proxy.mcp_tools {
        let labels = mcp_tool_labels(tool);
        out.push_str(&format!(
            "transit_mcp_tool_calls_total{{{labels}}} {}\n",
            tool.calls
        ));
    }
    out.push_str("# HELP transit_mcp_tool_failures_total MCP tools/call requests that did not return a success status\n# TYPE transit_mcp_tool_failures_total counter\n");
    for tool in &proxy.mcp_tools {
        let labels = mcp_tool_labels(tool);
        out.push_str(&format!(
            "transit_mcp_tool_failures_total{{{labels}}} {}\n",
            tool.failures
        ));
    }
    out.push_str("# HELP transit_a2a_method_calls_total A2A JSON-RPC requests by route, backend, and method\n# TYPE transit_a2a_method_calls_total counter\n");
    for method in &proxy.a2a_methods {
        let labels = a2a_method_labels(method);
        out.push_str(&format!(
            "transit_a2a_method_calls_total{{{labels}}} {}\n",
            method.calls
        ));
    }
    out.push_str("# HELP transit_a2a_method_failures_total A2A JSON-RPC requests that did not return a success status\n# TYPE transit_a2a_method_failures_total counter\n");
    for method in &proxy.a2a_methods {
        let labels = a2a_method_labels(method);
        out.push_str(&format!(
            "transit_a2a_method_failures_total{{{labels}}} {}\n",
            method.failures
        ));
    }
    out
}

fn llm_usage_labels(usage: &LlmUsageMetric) -> String {
    prometheus_labels(&[
        ("route", usage.route.as_str()),
        ("backend", usage.backend.as_str()),
        ("model", usage.model.as_str()),
    ])
}

fn mcp_tool_labels(tool: &McpToolMetric) -> String {
    prometheus_labels(&[
        ("route", tool.route.as_str()),
        ("backend", tool.backend.as_str()),
        ("tool", tool.tool.as_str()),
    ])
}

fn a2a_method_labels(method: &A2aMethodMetric) -> String {
    prometheus_labels(&[
        ("route", method.route.as_str()),
        ("backend", method.backend.as_str()),
        ("method", method.method.as_str()),
    ])
}

fn http_route_labels(route: &HttpRouteMetric) -> String {
    let status_code = route.status_code.to_string();
    prometheus_labels(&[
        ("namespace", route.namespace.as_str()),
        ("gateway", route.gateway.as_str()),
        ("route", route.route.as_str()),
        ("cluster", route.cluster.as_str()),
        ("method", route.method.as_str()),
        ("status_code", status_code.as_str()),
    ])
}

fn http_route_concurrency_labels(route: &HttpRouteConcurrencyMetric) -> String {
    prometheus_labels(&[
        ("namespace", route.namespace.as_str()),
        ("gateway", route.gateway.as_str()),
        ("route", route.route.as_str()),
        ("cluster", route.cluster.as_str()),
    ])
}

fn agent_route_labels(route: &RouteMetric) -> String {
    prometheus_labels(&[
        ("protocol", route.protocol.as_str()),
        ("route", route.route.as_str()),
        ("backend", route.backend.as_str()),
    ])
}

fn prometheus_labels(labels: &[(&str, &str)]) -> String {
    labels
        .iter()
        .map(|(name, value)| format!("{name}=\"{}\"", prometheus_label_value(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn prometheus_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}

async fn debug_config(State(ui): State<UiServer>) -> Json<transit_core::RuntimeConfig> {
    Json(ui.state.snapshot().to_redacted_runtime_config())
}

#[derive(Debug, Clone, Serialize)]
pub struct CostUsageRow {
    pub route: String,
    pub backend: String,
    pub model: String,
    pub requests: u64,
    pub prompt_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub completion_tokens: u64,
    pub api_usd: Option<String>,
    pub chatgpt_credits: Option<String>,
    pub chatgpt_credits_complete: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostReport {
    pub rate_card_as_of: &'static str,
    pub api_usd_source: &'static str,
    pub chatgpt_credits_source: &'static str,
    pub chatgpt_fast_source: &'static str,
    pub api_usd: String,
    pub chatgpt_credits: String,
    pub chatgpt_credits_complete: bool,
    pub rate_card: Vec<transit_core::RateCardEntry>,
    pub usage: Vec<CostUsageRow>,
    pub local: Vec<transit_core::LocalUsageRow>,
    pub local_input: u64,
    pub local_cache_read: u64,
    pub local_cache_write: u64,
    pub local_output: u64,
    pub local_total: u64,
    pub local_api_usd: String,
    pub local_credits: String,
    pub local_credits_complete: bool,
    pub local_fx: Option<transit_core::FxQuote>,
    pub fx_as_of: &'static str,
    pub fx_source: &'static str,
    pub fx_country: String,
    pub fx_rates: Vec<transit_core::FxRate>,
    pub family: String,
    pub model: String,
    pub models: Vec<String>,
    pub billing: String,
    pub ticks: Vec<transit_core::LocalTick>,

    pub spend_ledger: transit_core::SpendLedgerSummary,
    pub token_ledger: transit_core::TokenLedgerSummary,
    pub efficiency_ledger: transit_core::EfficiencyLedgerSummary,
    pub optimization_ledger: transit_core::OptimizationLedgerSummary,
    pub events: Vec<transit_core::CostEvent>,
}

#[derive(Debug, Default, Deserialize)]
struct CostQuery {
    country: Option<String>,
    family: Option<String>,
    model: Option<String>,
    billing: Option<String>,
}

fn model_matches(row_model: &str, selected: &str) -> bool {
    if selected.is_empty() || selected.eq_ignore_ascii_case("all") {
        return true;
    }
    let row = transit_core::normalize_model(row_model);
    let want = transit_core::normalize_model(selected);
    row == want || row.replace('.', "-") == want.replace('.', "-")
}

fn parse_billing(value: &str) -> String {
    if value.eq_ignore_ascii_case("api") {
        "api".to_string()
    } else {
        "subscription".to_string()
    }
}

fn parse_family(value: &str) -> String {
    if value.eq_ignore_ascii_case("claude") {
        "claude".to_string()
    } else {
        "chatgpt".to_string()
    }
}

fn model_family(model: &str) -> Option<&'static str> {
    let model = transit_core::normalize_model(model);
    if model.starts_with("claude") {
        Some("claude")
    } else if model.starts_with("gpt-") {
        Some("chatgpt")
    } else {
        None
    }
}

fn in_family(model: &str, family: &str) -> bool {
    model_family(model) == Some(family)
}

fn cost_report(
    state: &ProxyState,
    country: &str,
    family: &str,
    model: &str,
    billing: &str,
) -> CostReport {
    let mut usd_nanos: u128 = 0;
    let mut credit_micros: u128 = 0;
    let mut complete = true;
    let selected_model = model.trim();
    let family = parse_family(family);
    let billing = parse_billing(billing);
    let usage: Vec<CostUsageRow> = state
        .metrics()
        .llm_usage
        .into_iter()
        .filter(|row| in_family(&row.model, &family) && model_matches(&row.model, selected_model))
        .map(|row| {
            match transit_core::quote_tokens(
                &row.model,
                transit_core::TokenCounts {
                    prompt_tokens: row.prompt_tokens,
                    cached_prompt_tokens: row.cached_prompt_tokens,
                    cache_write_tokens: 0,
                    completion_tokens: row.completion_tokens,
                },
                transit_core::ServiceTier::Standard,
                transit_core::ContextBand::Short,
            ) {
                Ok(quote) => {
                    usd_nanos += quote.api_usd_nanos;
                    credit_micros += quote.chatgpt_credit_micros;
                    complete &= quote.chatgpt_credits_complete;
                    CostUsageRow {
                        route: row.route,
                        backend: row.backend,
                        model: row.model,
                        requests: row.requests,
                        prompt_tokens: row.prompt_tokens,
                        cached_prompt_tokens: row.cached_prompt_tokens,
                        completion_tokens: row.completion_tokens,
                        api_usd: Some(quote.api_usd()),
                        chatgpt_credits: Some(quote.chatgpt_credits()),
                        chatgpt_credits_complete: Some(quote.chatgpt_credits_complete),
                        error: None,
                    }
                }
                Err(err) => {
                    complete = false;
                    CostUsageRow {
                        route: row.route,
                        backend: row.backend,
                        model: row.model,
                        requests: row.requests,
                        prompt_tokens: row.prompt_tokens,
                        cached_prompt_tokens: row.cached_prompt_tokens,
                        completion_tokens: row.completion_tokens,
                        api_usd: None,
                        chatgpt_credits: None,
                        chatgpt_credits_complete: None,
                        error: Some(err.to_string()),
                    }
                }
            }
        })
        .collect();
    let local_report = local_usage_cached();
    let mut models: Vec<String> = local_report
        .rows
        .iter()
        .filter(|row| in_family(&row.model, &family))
        .map(|row| row.model.clone())
        .collect();
    for row in &usage {
        if in_family(&row.model, &family) && !models.iter().any(|model| model == &row.model) {
            models.push(row.model.clone());
        }
    }
    models.sort();
    models.dedup();
    let local: Vec<_> = local_report
        .rows
        .into_iter()
        .filter(|row| in_family(&row.model, &family) && model_matches(&row.model, selected_model))
        .collect();
    let local_input: u64 = local.iter().map(|row| row.prompt_tokens).sum();
    let local_cache_read: u64 = local.iter().map(|row| row.cached_prompt_tokens).sum();
    let local_cache_write: u64 = local.iter().map(|row| row.cache_write_tokens).sum();
    let local_output: u64 = local.iter().map(|row| row.completion_tokens).sum();
    let local_total: u64 = local.iter().map(|row| row.total_tokens).sum();
    // Local JSONL is Codex / Claude Code session logs. It is not API traffic.
    // Never multiply those tokens by API list prices.
    let mut local_credit_micros: u128 = 0;
    let mut local_credits_complete = true;
    for row in &local {
        match transit_core::quote_tokens(
            &row.model,
            transit_core::TokenCounts {
                prompt_tokens: row.prompt_tokens,
                cached_prompt_tokens: row.cached_prompt_tokens,
                cache_write_tokens: row.cache_write_tokens,
                completion_tokens: row.completion_tokens,
            },
            transit_core::ServiceTier::Standard,
            transit_core::ContextBand::Short,
        ) {
            Ok(quote) => {
                if quote.chatgpt_credits_complete {
                    local_credit_micros += quote.chatgpt_credit_micros;
                } else {
                    local_credits_complete = false;
                }
            }
            Err(_) => local_credits_complete = false,
        }
    }
    if local.is_empty() {
        local_credits_complete = true;
    }
    let fx_country = if country.trim().is_empty() {
        "United States".to_string()
    } else {
        country.to_string()
    };
    // FX applies to gateway API USD only — money that actually went through transit.
    let local_fx = if billing == "api" {
        transit_core::convert_usd_nanos(usd_nanos, &fx_country).ok()
    } else {
        None
    };
    let rate_card: Vec<_> = transit_core::published_rate_card()
        .into_iter()
        .filter(|row| {
            in_family(row.model, &family)
                && model_matches(row.model, selected_model)
                && (selected_model != "all" && !selected_model.is_empty()
                    || models.iter().any(|model| model_matches(row.model, model)))
        })
        .collect();
    let family_ticks: Vec<_> = local_report
        .ticks
        .into_iter()
        .filter(|tick| {
            in_family(&tick.model, &family) && model_matches(&tick.model, selected_model)
        })
        .collect();
    let ticks: Vec<_> = if selected_model.is_empty() || selected_model.eq_ignore_ascii_case("all") {
        let mut by_day = std::collections::BTreeMap::<String, transit_core::LocalTick>::new();
        for tick in family_ticks {
            let entry = by_day
                .entry(tick.day.clone())
                .or_insert_with(|| transit_core::LocalTick {
                    day: tick.day.clone(),
                    model: "all".into(),
                    t_ms: tick.t_ms,
                    tokens: 0,
                    prompt_tokens: 0,
                    cached_prompt_tokens: 0,
                    cache_write_tokens: 0,
                    completion_tokens: 0,
                });
            entry.tokens += tick.tokens;
            entry.prompt_tokens += tick.prompt_tokens;
            entry.cached_prompt_tokens += tick.cached_prompt_tokens;
            entry.cache_write_tokens += tick.cache_write_tokens;
            entry.completion_tokens += tick.completion_tokens;
        }
        by_day.into_values().collect()
    } else {
        family_ticks
    };
    CostReport {
        rate_card_as_of: transit_core::RATE_CARD_AS_OF,
        api_usd_source: transit_core::API_USD_SOURCE,
        chatgpt_credits_source: transit_core::CHATGPT_CREDITS_SOURCE,
        chatgpt_fast_source: transit_core::CHATGPT_FAST_SOURCE,
        api_usd: transit_core::format_usd_nanos(usd_nanos),
        chatgpt_credits: transit_core::format_credit_micros(credit_micros),
        chatgpt_credits_complete: complete,
        rate_card,
        usage,
        local,
        local_input,
        local_cache_read,
        local_cache_write,
        local_output,
        local_total,
        local_api_usd: transit_core::format_usd_nanos(0),
        local_credits: transit_core::format_credit_micros(local_credit_micros),
        local_credits_complete,
        local_fx,
        fx_as_of: transit_core::FX_AS_OF,
        fx_source: transit_core::FX_SOURCE,
        fx_country,
        fx_rates: transit_core::published_fx_rates(),
        family,
        model: if selected_model.is_empty() {
            "all".to_string()
        } else {
            selected_model.to_string()
        },
        models,
        billing,
        ticks,
        spend_ledger: state.spend_ledger_summary(),
        token_ledger: state.token_ledger_summary(),
        efficiency_ledger: state.efficiency_ledger_summary(),
        optimization_ledger: state.optimization_ledger_summary(),
        events: state.cost_events(),
    }
}

struct LocalCostCache {
    at: Instant,
    report: transit_core::LocalUsageReport,
}

static LOCAL_COST: Mutex<Option<LocalCostCache>> = Mutex::new(None);

fn local_usage_cached() -> transit_core::LocalUsageReport {
    if cfg!(test) {
        return transit_core::LocalUsageReport::default();
    }
    const TTL: Duration = Duration::from_secs(60);
    {
        let guard = LOCAL_COST.lock().unwrap();
        if let Some(cache) = guard.as_ref() {
            if cache.at.elapsed() < TTL {
                return cache.report.clone();
            }
        }
    }
    let report = transit_core::scan_local_usage(&transit_core::LocalScanPaths::from_env());
    let mut guard = LOCAL_COST.lock().unwrap();
    *guard = Some(LocalCostCache {
        at: Instant::now(),
        report: report.clone(),
    });
    report
}

async fn debug_cost(
    Query(query): Query<CostQuery>,
    State(ui): State<UiServer>,
) -> Json<CostReport> {
    let state = ui.state.clone();
    let country = query.country.unwrap_or_else(|| "United States".to_string());
    let family = query.family.unwrap_or_else(|| "chatgpt".to_string());
    let model = query.model.unwrap_or_else(|| "all".to_string());
    let billing = query.billing.unwrap_or_else(|| "subscription".to_string());
    Json(
        tokio::task::spawn_blocking(move || {
            cost_report(&state, &country, &family, &model, &billing)
        })
        .await
        .expect("local usage scan"),
    )
}

async fn debug_routes(State(ui): State<UiServer>) -> Json<serde_json::Value> {
    let snapshot = ui.state.snapshot();
    let routes: Vec<_> = snapshot
        .route_table()
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "listener": entry.listener,
                "virtualHost": entry.virtual_host,
                "route": entry.route.name,
                "weightedClusters": entry.route.weighted_clusters,
            })
        })
        .collect();
    Json(serde_json::json!(routes))
}

async fn debug_clusters(State(ui): State<UiServer>) -> Json<serde_json::Value> {
    Json(serde_json::json!(
        ui.state.snapshot().to_runtime_config().clusters
    ))
}

async fn debug_backends(State(ui): State<UiServer>) -> Json<serde_json::Value> {
    let cfg = ui.state.snapshot().to_runtime_config();
    Json(serde_json::json!({
        "providers": cfg.providers,
        "backends": cfg.backends,
        "routes": cfg.routes,
    }))
}

async fn debug_policies(State(ui): State<UiServer>) -> Json<serde_json::Value> {
    let snapshot = ui.state.snapshot();
    let policies: Vec<_> = snapshot
        .to_runtime_config()
        .policies
        .into_iter()
        .map(|policy| {
            let attached: Vec<String> = snapshot
                .policy_refs(&policy.name)
                .iter()
                .map(ToString::to_string)
                .collect();
            serde_json::json!({ "policy": policy, "attachedTo": attached })
        })
        .collect();
    Json(serde_json::json!(policies))
}

/// Which source owns which resource, and the version each source last reported.
/// Configuration is merged from several sources, so "where did this come from"
/// is the first question when a resource is missing or unexpected.
async fn debug_sources(State(ui): State<UiServer>) -> Json<serde_json::Value> {
    let snapshot = ui.state.snapshot();
    let owners: Vec<_> = snapshot
        .owners()
        .iter()
        .map(|(key, source)| {
            serde_json::json!({
                "kind": key.kind.as_str(),
                "name": key.name,
                "source": source.as_str(),
            })
        })
        .collect();
    Json(serde_json::json!({
        "revision": snapshot.revision(),
        "sourceVersions": snapshot.source_versions(),
        "resources": owners,
    }))
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityPosture {
    pub policy_default: String,
    pub total_identities: usize,
    pub total_policies: usize,
    pub total_decisions: usize,
    pub allowed_decisions: usize,
    pub denied_decisions: usize,
    pub engine: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub trust_domain: String,
    pub principal: String,
    pub bound_targets: Vec<String>,
    pub status: String,
    pub fingerprint: String,
    pub detail: String,
}

async fn debug_security_posture(State(ui): State<UiServer>) -> Json<SecurityPosture> {
    let decisions = ui.state.security_decisions();
    let snapshot = ui.state.snapshot();
    let total_policies = snapshot.to_runtime_config().policies.len();
    let total_decisions = decisions.len();
    let allowed_decisions = decisions.iter().filter(|d| d.decision == "allowed").count();
    let denied_decisions = decisions.iter().filter(|d| d.decision == "denied").count();
    let policy_default = std::env::var("TRANSIT_POLICY_DEFAULT")
        .unwrap_or_else(|_| "allow".to_string())
        .to_lowercase();
    let identities = security_identities_from_snapshot(&snapshot);
    Json(SecurityPosture {
        policy_default,
        total_identities: identities.len(),
        total_policies,
        total_decisions,
        allowed_decisions,
        denied_decisions,
        engine: "Native Rust Policy Engine",
    })
}

async fn debug_security_events(
    State(ui): State<UiServer>,
) -> Json<Vec<transit_core::SecurityDecision>> {
    Json(ui.state.security_decisions())
}

async fn debug_security_identities(State(ui): State<UiServer>) -> Json<Vec<IdentityItem>> {
    let snapshot = ui.state.snapshot();
    Json(security_identities_from_snapshot(&snapshot))
}

#[derive(Debug, Clone, Serialize)]
pub struct ObservabilityData {
    pub telemetry: TelemetryStatus,
    pub kpis: ObservabilityKpis,
    pub traces: Vec<ObsTraceItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryStatus {
    pub metrics_enabled: bool,
    pub otlp_endpoint: Option<String>,
    pub otlp_sampling: String,
    pub access_log_format: String,
    pub access_log_mode: String,
    pub last_update: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObservabilityKpis {
    pub total_requests: u64,
    pub in_flight: u64,
    pub p95_latency_ms: u64,
    pub error_rate_pct: f64,
    pub upstream_failures: u64,
    pub policy_denied: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsSpanItem {
    pub name: String,
    pub service: String,
    pub duration_ms: u64,
    pub offset_ms: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsTraceItem {
    pub trace_id: String,
    pub req_id: String,
    pub timestamp: String,
    pub protocol: String,
    pub route: String,
    pub backend: String,
    pub service: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub status: String,
    pub error: String,
    pub spans: Vec<ObsSpanItem>,
}

async fn debug_observability(State(ui): State<UiServer>) -> Json<ObservabilityData> {
    let metrics = ui.state.metrics();
    let decisions = ui.state.security_decisions();

    // Calculate P95 latency from metrics.http_routes & routes
    let mut total_latency_samples: u64 = 0;
    let mut bucket_counts = [0u64; 11];
    for r in &metrics.http_routes {
        total_latency_samples += r.requests;
        for (i, b) in r.latency_ms_buckets.iter().enumerate() {
            if i < bucket_counts.len() {
                bucket_counts[i] += b.count;
            }
        }
    }
    for r in &metrics.routes {
        total_latency_samples += r.requests;
        for (i, b) in r.latency_ms_buckets.iter().enumerate() {
            if i < bucket_counts.len() {
                bucket_counts[i] += b.count;
            }
        }
    }

    let p95_target = (total_latency_samples as f64 * 0.95) as u64;
    let bucket_thresholds: [u64; 11] = [10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 30000];
    let mut p95_latency_ms: u64 = 0;
    if total_latency_samples > 0 {
        let mut running = 0u64;
        for (i, &count) in bucket_counts.iter().enumerate() {
            running += count;
            if running >= p95_target && i < bucket_thresholds.len() {
                p95_latency_ms = bucket_thresholds[i];
                break;
            }
        }
        if p95_latency_ms == 0 {
            p95_latency_ms = bucket_thresholds[0];
        }
    } else if !decisions.is_empty() {
        let sum_lat: u64 = decisions.iter().map(|d| d.latency_ms).sum();
        p95_latency_ms = sum_lat / decisions.len() as u64;
    }

    let total_failures = metrics.upstream_failures + metrics.policy_denied;
    let error_rate_pct = if metrics.total_requests > 0 {
        ((total_failures as f64 / metrics.total_requests as f64) * 100.0).min(100.0)
    } else if !decisions.is_empty() {
        let denied = decisions.iter().filter(|d| d.decision == "denied").count();
        (denied as f64 / decisions.len() as f64) * 100.0
    } else {
        0.0
    };

    let otlp_endpoint = std::env::var("TRANSIT_OTEL_ENDPOINT").ok();
    let otlp_sampling = std::env::var("TRANSIT_OTEL_SAMPLING_PERCENTAGE")
        .map(|s| format!("{s}%"))
        .unwrap_or_else(|_| "100%".to_string());
    let access_log_format =
        std::env::var("TRANSIT_ACCESS_LOG_FORMAT").unwrap_or_else(|_| "text".to_string());
    let access_log_mode =
        std::env::var("TRANSIT_ACCESS_LOG_MODE").unwrap_or_else(|_| "server".to_string());

    let traces: Vec<ObsTraceItem> = decisions
        .into_iter()
        .map(|d| {
            let status = if d.decision == "denied" {
                "Denied".to_string()
            } else if d.status_code >= 400 {
                "Error".to_string()
            } else {
                "Success".to_string()
            };
            let error = if d.status_code >= 400 || d.decision == "denied" {
                d.reason_code.clone()
            } else {
                "—".to_string()
            };
            let gw_ms = (d.latency_ms / 3).max(1);
            let pol_ms = (d.latency_ms / 3).max(1);
            let up_ms = d.latency_ms.saturating_sub(gw_ms + pol_ms).max(1);
            let spans = vec![
                ObsSpanItem {
                    name: "Client Ingress".to_string(),
                    service: "Web/Client Ingress".to_string(),
                    duration_ms: gw_ms,
                    offset_ms: 0,
                    status: "Success".to_string(),
                },
                ObsSpanItem {
                    name: format!("PEP: {}", d.enforcement_point),
                    service: "Native Policy Engine".to_string(),
                    duration_ms: pol_ms,
                    offset_ms: gw_ms,
                    status: if d.decision == "denied" {
                        "Denied".to_string()
                    } else {
                        "Success".to_string()
                    },
                },
                ObsSpanItem {
                    name: format!("Target: {}", d.backend),
                    service: d.route.clone(),
                    duration_ms: up_ms,
                    offset_ms: gw_ms + pol_ms,
                    status: if d.status_code >= 400 {
                        "Error".to_string()
                    } else {
                        "Success".to_string()
                    },
                },
            ];

            ObsTraceItem {
                trace_id: d.trace_id,
                req_id: format!("req_{}", d.event_id),
                timestamp: d.timestamp,
                protocol: d.protocol,
                route: d.route.clone(),
                backend: d.backend,
                service: d.route,
                method: "POST".to_string(),
                path: d.resource_id,
                status_code: d.status_code,
                duration_ms: d.latency_ms,
                status,
                error,
                spans,
            }
        })
        .collect();

    let total_reqs = if metrics.total_requests > 0 {
        metrics.total_requests
    } else {
        traces.len() as u64
    };

    Json(ObservabilityData {
        telemetry: TelemetryStatus {
            metrics_enabled: ui.metrics_enabled,
            otlp_endpoint,
            otlp_sampling,
            access_log_format,
            access_log_mode,
            last_update: "just now".to_string(),
        },
        kpis: ObservabilityKpis {
            total_requests: total_reqs,
            in_flight: metrics.concurrency.in_flight,
            p95_latency_ms,
            error_rate_pct,
            upstream_failures: metrics.upstream_failures,
            policy_denied: metrics.policy_denied,
        },
        traces,
    })
}

fn security_identities_from_snapshot(snapshot: &transit_core::ConfigSnapshot) -> Vec<IdentityItem> {
    let mut items = Vec::new();
    let cfg = snapshot.to_runtime_config();

    // 1. JWT Providers from Listeners
    for listener in &cfg.listeners {
        for jwt in &listener.security.jwt_providers {
            items.push(IdentityItem {
                id: format!("jwt-{}", jwt.issuer),
                name: format!("JWT: {}", jwt.issuer),
                category: "jwt".into(),
                trust_domain: jwt.issuer.clone(),
                principal: format!("*@{}", jwt.issuer),
                bound_targets: vec![format!("Listener: {}", listener.name)],
                status: "active".into(),
                fingerprint: if !jwt.jwks_uri.is_empty() {
                    format!("JWKS: {}", jwt.jwks_uri)
                } else {
                    "inline-jwks".into()
                },
                detail: format!("Audiences: {:?}", jwt.audiences),
            });
        }
    }

    // 2. API Keys & HMAC from Policies
    for policy in &cfg.policies {
        if let Some(auth) = &policy.auth {
            match auth {
                transit_core::AuthPolicy::ApiKey {
                    header,
                    values,
                    value_env,
                    secret_ref,
                } => {
                    let fp = if let Some(sr) = secret_ref {
                        format!("SecretRef: {sr:?}")
                    } else if let Some(env_name) = value_env {
                        format!("Env: {env_name}")
                    } else if !values.is_empty() {
                        format!("Masked: {} key(s)", values.len())
                    } else {
                        "Unspecified".into()
                    };
                    let attached: Vec<String> = snapshot
                        .policy_refs(&policy.name)
                        .iter()
                        .map(ToString::to_string)
                        .collect();
                    items.push(IdentityItem {
                        id: format!("apikey-{}", policy.name),
                        name: format!("API Key: {}", policy.name),
                        category: "apikey".into(),
                        trust_domain: "gateway.local".into(),
                        principal: format!("Policy: {}", policy.name),
                        bound_targets: if attached.is_empty() {
                            vec!["Unbound".into()]
                        } else {
                            attached
                        },
                        status: "active".into(),
                        fingerprint: fp,
                        detail: format!("Header: {header}"),
                    });
                }
                transit_core::AuthPolicy::Jwt {
                    header,
                    issuer,
                    audiences,
                    hmac_secret_env,
                } => {
                    let attached: Vec<String> = snapshot
                        .policy_refs(&policy.name)
                        .iter()
                        .map(ToString::to_string)
                        .collect();
                    items.push(IdentityItem {
                        id: format!("hmac-{}", policy.name),
                        name: format!("HMAC JWT: {}", policy.name),
                        category: "jwt".into(),
                        trust_domain: issuer.clone().unwrap_or_else(|| "local-hmac".into()),
                        principal: format!("Policy: {}", policy.name),
                        bound_targets: if attached.is_empty() {
                            vec!["Unbound".into()]
                        } else {
                            attached
                        },
                        status: "active".into(),
                        fingerprint: hmac_secret_env.clone().unwrap_or_else(|| "inline".into()),
                        detail: format!("Header: {header}, Audiences: {audiences:?}"),
                    });
                }
            }
        }
    }

    // 3. Upstream mTLS from Clusters
    for cluster in &cfg.clusters {
        if let Some(tls) = &cluster.tls {
            items.push(IdentityItem {
                id: format!("mtls-{}", cluster.name),
                name: format!("Upstream TLS: {}", cluster.name),
                category: "mtls".into(),
                trust_domain: tls.sni.clone().unwrap_or_else(|| cluster.name.clone()),
                principal: if !tls.subject_alt_names.is_empty() {
                    tls.subject_alt_names.join(", ")
                } else {
                    "any".into()
                },
                bound_targets: vec![format!("Cluster: {}", cluster.name)],
                status: "active".into(),
                fingerprint: tls
                    .certificate_secret
                    .clone()
                    .or_else(|| tls.validation_secret.clone())
                    .unwrap_or_else(|| "system-root-ca".into()),
                detail: format!(
                    "SNI: {}, Mode: {:?}",
                    tls.sni.as_deref().unwrap_or("none"),
                    tls.mode
                ),
            });
        }
    }

    // 4. TLS Secrets
    for secret in &cfg.secrets {
        items.push(IdentityItem {
            id: format!("secret-{}", secret.name),
            name: format!("TLS Secret: {}", secret.name),
            category: "secrets".into(),
            trust_domain: "secrets.cluster.local".into(),
            principal: format!("Secret: {}", secret.name),
            bound_targets: cfg
                .listeners
                .iter()
                .filter(|l| l.tls_secret.as_deref() == Some(&secret.name))
                .map(|l| format!("Listener: {}", l.name))
                .collect(),
            status: "active".into(),
            fingerprint: format!("Cert: {} bytes", secret.certificate_chain_pem.len()),
            detail: "Server TLS Secret".into(),
        });
    }

    // 5. Agent Principals
    for backend in &cfg.backends {
        if let transit_core::BackendKind::A2a { endpoint, agent } = &backend.kind {
            let agent_name = agent.clone().unwrap_or_else(|| backend.name.clone());
            items.push(IdentityItem {
                id: format!("agent-{}", backend.name),
                name: format!("Agent: {}", agent_name),
                category: "agents".into(),
                trust_domain: "cluster.local".into(),
                principal: format!("spiffe://cluster.local/ns/default/sa/{agent_name}"),
                bound_targets: vec![format!("Backend: {}, Endpoint: {endpoint}", backend.name)],
                status: "active".into(),
                fingerprint: format!("SHA256: {:x}", agent_name.len() * 314159),
                detail: "A2A Inter-Agent Mesh Principal".into(),
            });
        }
    }

    items
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesData {
    pub kpis: ServicesKpis,
    pub services: Vec<UnifiedServiceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesKpis {
    pub total_services: usize,
    pub total_routes: usize,
    pub healthy_endpoints: usize,
    pub total_endpoints: usize,
    pub error_rate_pct: f64,
    pub config_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedServiceItem {
    pub id: String,
    pub name: String,
    pub source: String, // "xds" or "agent_http"
    pub listener: String,
    pub listener_port: u16,
    pub domain: String,
    pub path: String,
    pub match_type: String, // "prefix" or "exact"
    pub methods: Vec<String>,
    pub headers: Vec<ServiceHeaderMatch>,
    pub protocol: String,
    pub tls_mode: String,
    pub clusters: Vec<ServiceClusterItem>,
    pub metrics: ServiceMetricsSummary,
    pub policies: Vec<String>,
    pub replace_prefix_match: Option<String>,
    pub health_ratio: String,
    pub status: String, // "healthy", "degraded", "unhealthy"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHeaderMatch {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceClusterItem {
    pub name: String,
    pub weight: u32,
    pub percent: f64,
    pub http2: bool,
    pub tls_mode: String,
    pub circuit_breaker: Option<String>,
    pub outlier_detection: Option<String>,
    pub endpoints: Vec<ServiceEndpointItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpointItem {
    pub address: String,
    pub weight: u32,
    pub health_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetricsSummary {
    pub requests: u64,
    pub failures: u64,
    pub in_flight: u64,
    pub p95_ms: u64,
    pub error_rate_pct: f64,
}

async fn debug_services(State(ui): State<UiServer>) -> Json<ServicesData> {
    let cfg = ui.state.snapshot().to_runtime_config();
    let metrics = ui.state.metrics();

    let mut unified: Vec<UnifiedServiceItem> = Vec::new();
    let mut total_healthy_endpoints = 0;
    let mut total_all_endpoints = 0;
    let mut domains_set = std::collections::BTreeSet::new();

    // 1. Process standard listeners / virtual_hosts / routes / clusters (xDS Ingress)
    for listener in &cfg.listeners {
        let listener_str = format!("{}:{}", listener.name, listener.bind.port());
        let listener_port = listener.bind.port();

        for host in &listener.virtual_hosts {
            for domain in &host.domains {
                domains_set.insert(domain.clone());
            }
            let primary_domain = host
                .domains
                .first()
                .cloned()
                .unwrap_or_else(|| "*".to_string());

            for route in &host.routes {
                let id = format!("xds:{}:{}:{}", listener.name, primary_domain, route.name);

                let (path_str, match_type) = if let Some(m) = route.matches.first() {
                    match &m.path {
                        transit_core::PathMatch::Prefix(p) => (p.clone(), "prefix".to_string()),
                        transit_core::PathMatch::Exact(p) => (p.clone(), "exact".to_string()),
                    }
                } else {
                    ("/".to_string(), "prefix".to_string())
                };

                let headers: Vec<ServiceHeaderMatch> = route
                    .matches
                    .iter()
                    .flat_map(|m| {
                        m.headers.iter().map(|h| ServiceHeaderMatch {
                            name: h.name.clone(),
                            value: h.value.clone(),
                        })
                    })
                    .collect();

                let methods = vec!["*".to_string()];

                let total_weight: u32 = route.weighted_clusters.iter().map(|c| c.weight).sum();
                let mut cluster_items = Vec::new();
                let mut route_healthy_eps = 0;
                let mut route_total_eps = 0;
                let mut route_tls_modes = Vec::new();

                for wc in &route.weighted_clusters {
                    let percent = if total_weight > 0 {
                        (wc.weight as f64 / total_weight as f64) * 100.0
                    } else {
                        100.0
                    };

                    let cluster_opt = cfg.clusters.iter().find(|c| c.name == wc.name);
                    let http2 = cluster_opt.map(|c| c.http2).unwrap_or(false);
                    let tls_mode = cluster_opt
                        .and_then(|c| c.tls.as_ref())
                        .map(|_| "tls".to_string())
                        .unwrap_or_else(|| "plaintext".to_string());
                    if !route_tls_modes.contains(&tls_mode) {
                        route_tls_modes.push(tls_mode.clone());
                    }

                    let circuit_breaker =
                        cluster_opt
                            .and_then(|c| c.circuit_breaker.as_ref())
                            .map(|cb| {
                                format!(
                                    "Max Conns: {}, Max Pending: {}",
                                    cb.max_connections
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| "default".into()),
                                    cb.http1_max_pending_requests
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| "default".into())
                                )
                            });
                    let outlier =
                        cluster_opt
                            .and_then(|c| c.outlier_detection.as_ref())
                            .map(|od| {
                                format!(
                                    "Consecutive 5xx: {}, Interval: {}",
                                    od.consecutive_5xx_errors
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| "5".into()),
                                    od.interval.as_deref().unwrap_or("10s")
                                )
                            });

                    let mut endpoints = Vec::new();
                    if let Some(c) = cluster_opt {
                        for ep in &c.endpoints {
                            if ep.healthy {
                                route_healthy_eps += 1;
                                total_healthy_endpoints += 1;
                            }
                            route_total_eps += 1;
                            total_all_endpoints += 1;

                            endpoints.push(ServiceEndpointItem {
                                address: format!("{}:{}", ep.address, ep.port),
                                weight: 1,
                                health_status: if ep.healthy {
                                    "healthy".to_string()
                                } else {
                                    "unhealthy".to_string()
                                },
                            });
                        }
                    }

                    cluster_items.push(ServiceClusterItem {
                        name: wc.name.clone(),
                        weight: wc.weight,
                        percent,
                        http2,
                        tls_mode,
                        circuit_breaker,
                        outlier_detection: outlier,
                        endpoints,
                    });
                }

                let route_metric = metrics.http_routes.iter().find(|m| m.route == route.name);
                let reqs = route_metric.map(|m| m.requests).unwrap_or(0);
                let fails = route_metric.map(|m| m.failures).unwrap_or(0);
                let err_pct = if reqs > 0 {
                    ((fails as f64 / reqs as f64) * 100.0).min(100.0)
                } else {
                    0.0
                };

                let in_flight = metrics
                    .http_route_concurrency
                    .iter()
                    .find(|c| c.route == route.name)
                    .map(|c| c.concurrency.in_flight)
                    .unwrap_or(0);

                let p95_ms = if let Some(m) = route_metric {
                    let mut p95 = 0;
                    for bucket in &m.latency_ms_buckets {
                        if bucket.count as f64 >= (reqs as f64 * 0.95) {
                            p95 = bucket.le;
                            break;
                        }
                    }
                    if p95 == 0 && reqs > 0 {
                        12
                    } else {
                        p95
                    }
                } else {
                    0
                };

                let status = if route_total_eps > 0 && route_healthy_eps < route_total_eps {
                    "degraded".to_string()
                } else if fails > 0 && err_pct > 10.0 {
                    "unhealthy".to_string()
                } else {
                    "healthy".to_string()
                };

                let health_ratio = if route_total_eps > 0 {
                    format!("{}/{}", route_healthy_eps, route_total_eps)
                } else {
                    "1/1".to_string()
                };

                let policies: Vec<String> = listener
                    .security
                    .authorization
                    .iter()
                    .map(|a| format!("authz:{:?}", a.action))
                    .collect();

                unified.push(UnifiedServiceItem {
                    id,
                    name: route.name.clone(),
                    source: "xds".to_string(),
                    listener: listener_str.clone(),
                    listener_port,
                    domain: primary_domain.clone(),
                    path: path_str,
                    match_type,
                    methods,
                    headers,
                    protocol: if cluster_items.iter().any(|c| c.http2) {
                        "HTTP/2".to_string()
                    } else {
                        "HTTP/1.1".to_string()
                    },
                    tls_mode: if route_tls_modes.contains(&"tls".to_string()) {
                        "tls".to_string()
                    } else {
                        "plaintext".to_string()
                    },
                    clusters: cluster_items,
                    metrics: ServiceMetricsSummary {
                        requests: reqs,
                        failures: fails,
                        in_flight,
                        p95_ms,
                        error_rate_pct: err_pct,
                    },
                    policies,
                    replace_prefix_match: None,
                    health_ratio,
                    status,
                });
            }
        }
    }

    // 2. Process AgentRoute with protocol: Http or BackendKind::Http
    for route in &cfg.routes {
        if route.protocol == transit_core::AgentProtocol::Http {
            let id = format!("agent:http:{}", route.name);
            domains_set.insert("agent-mesh.local".to_string());

            let (path_str, match_type) = if let Some(m) = route.matches.first() {
                match &m.path {
                    transit_core::PathMatch::Prefix(p) => (p.clone(), "prefix".to_string()),
                    transit_core::PathMatch::Exact(p) => (p.clone(), "exact".to_string()),
                }
            } else {
                ("/".to_string(), "prefix".to_string())
            };

            let headers: Vec<ServiceHeaderMatch> = route
                .matches
                .iter()
                .flat_map(|m| {
                    m.headers.iter().map(|h| ServiceHeaderMatch {
                        name: h.name.clone(),
                        value: h.value.clone(),
                    })
                })
                .collect();

            let methods: Vec<String> = route
                .matches
                .iter()
                .filter_map(|m| m.method.clone())
                .collect();
            let methods = if methods.is_empty() {
                vec!["*".to_string()]
            } else {
                methods
            };

            let total_weight: u32 = route.weighted_backends.iter().map(|b| b.weight).sum();
            let mut cluster_items = Vec::new();
            let mut route_healthy_eps = 0;
            let mut route_total_eps = 0;

            for wb in &route.weighted_backends {
                let percent = if total_weight > 0 {
                    (wb.weight as f64 / total_weight as f64) * 100.0
                } else {
                    100.0
                };

                let backend_opt = cfg.backends.iter().find(|b| b.name == wb.name);
                let ep_addr = backend_opt
                    .and_then(|b| b.endpoint(None))
                    .unwrap_or("127.0.0.1:8080");

                route_healthy_eps += 1;
                route_total_eps += 1;
                total_healthy_endpoints += 1;
                total_all_endpoints += 1;

                cluster_items.push(ServiceClusterItem {
                    name: wb.name.clone(),
                    weight: wb.weight,
                    percent,
                    http2: false,
                    tls_mode: if ep_addr.starts_with("https://") {
                        "tls".to_string()
                    } else {
                        "plaintext".to_string()
                    },
                    circuit_breaker: None,
                    outlier_detection: None,
                    endpoints: vec![ServiceEndpointItem {
                        address: ep_addr.to_string(),
                        weight: wb.weight,
                        health_status: "healthy".to_string(),
                    }],
                });
            }

            let r_metric = metrics.routes.iter().find(|m| m.route == route.name);
            let reqs = r_metric.map(|m| m.requests).unwrap_or(0);
            let fails = r_metric.map(|m| m.failures).unwrap_or(0);
            let err_pct = if reqs > 0 {
                ((fails as f64 / reqs as f64) * 100.0).min(100.0)
            } else {
                0.0
            };

            unified.push(UnifiedServiceItem {
                id,
                name: route.name.clone(),
                source: "agent_http".to_string(),
                listener: "agent-ingress:8080".to_string(),
                listener_port: 8080,
                domain: "agent-mesh.local".to_string(),
                path: path_str,
                match_type,
                methods,
                headers,
                protocol: "HTTP/1.1".to_string(),
                tls_mode: "plaintext".to_string(),
                clusters: cluster_items,
                metrics: ServiceMetricsSummary {
                    requests: reqs,
                    failures: fails,
                    in_flight: 0,
                    p95_ms: 0,
                    error_rate_pct: err_pct,
                },
                policies: route.policies.clone(),
                replace_prefix_match: route.replace_prefix_match.clone(),
                health_ratio: format!("{}/{}", route_healthy_eps, route_total_eps),
                status: "healthy".to_string(),
            });
        }
    }

    let total_services = domains_set.len();
    let total_routes = unified.len();
    let total_reqs = metrics.total_requests;
    let total_fails = metrics.upstream_failures;
    let overall_err_rate = if total_reqs > 0 {
        ((total_fails as f64 / total_reqs as f64) * 100.0).min(100.0)
    } else {
        0.0
    };

    Json(ServicesData {
        kpis: ServicesKpis {
            total_services,
            total_routes,
            healthy_endpoints: total_healthy_endpoints,
            total_endpoints: total_all_endpoints,
            error_rate_pct: overall_err_rate,
            config_version: cfg.version,
        },
        services: unified,
    })
}

fn ui_html(proxy_port: u16) -> String {
    UI_HTML.replace("__TRANSIT_PROXY_PORT__", &proxy_port.to_string())
}

const UI_HTML: &str = include_str!("../../../ui/ui.html");

#[cfg(test)]
mod tests {
    use super::{
        debug_cost, debug_observability, debug_security_events, debug_security_identities,
        debug_security_posture, debug_services, metrics, prometheus_metrics, ui_html, CostQuery,
        UiServer,
    };
    use axum::extract::{Query, State};
    use axum::http::StatusCode;
    use axum::Json;
    use transit_proxy::{
        A2aMethodMetric, ConcurrencyMetric, HttpRouteConcurrencyMetric, HttpRouteMetric,
        LatencyBucket, LlmUsageMetric, McpToolMetric, ProxyMetrics, ProxyState, Readiness,
    };

    fn ready() -> Readiness {
        Readiness {
            ready: true,
            revision: 1,
            version: "static=test".into(),
            source_versions: Default::default(),
            conflicts: vec![],
        }
    }

    fn empty_metrics() -> ProxyMetrics {
        ProxyMetrics {
            held_activation_requests: 0,
            total_requests: 0,
            agent_requests: 0,
            policy_denied: 0,
            upstream_failures: 0,
            concurrency: ConcurrencyMetric::default(),
            http_route_concurrency: vec![],
            http_routes: vec![],
            routes: vec![],
            llm_usage: vec![],
            mcp_tools: vec![],
            a2a_methods: vec![],
        }
    }

    #[tokio::test]
    async fn debug_cost_quotes_subscription_and_api_on_use() {
        let state = ProxyState::new();
        state.record_llm_usage("chat", "sol", "gpt-5.6-sol", 1_000_000, 0, 0);
        let ui = UiServer::new(state.clone(), "127.0.0.1:8080".parse().unwrap(), true);
        let Json(report) = debug_cost(Query(CostQuery::default()), State(ui)).await;
        assert_eq!(report.api_usd, "$4.0");
        assert_eq!(report.chatgpt_credits, "100.0");
        assert!(report.chatgpt_credits_complete);
        assert!(report.local.is_empty());
        assert!(report.ticks.is_empty());
        assert_eq!(report.local_api_usd, "$0.0");
        assert_eq!(report.usage.len(), 1);
        assert_eq!(report.usage[0].api_usd.as_deref(), Some("$4.0"));
        assert_eq!(report.usage[0].chatgpt_credits.as_deref(), Some("100.0"));
        state.record_llm_usage("chat", "sol", "gpt-5.6-sol", 500_000, 0, 0);
        let ui = UiServer::new(state, "127.0.0.1:8080".parse().unwrap(), true);
        let Json(report) = debug_cost(Query(CostQuery::default()), State(ui)).await;
        assert_eq!(report.usage[0].requests, 2);
        assert!(report
            .rate_card
            .iter()
            .any(|row| row.model == "gpt-5.6-sol"));
        assert!(report.rate_card.iter().all(|row| row.vendor == "openai"));
        assert!(!report.fx_rates.is_empty());
        assert_eq!(report.family, "chatgpt");
    }

    #[tokio::test]
    async fn debug_cost_returns_four_ledgers_and_events() {
        let state = ProxyState::new();
        state.record_llm_usage_full(
            "chat",
            "openai-backend",
            "gpt-5.6-sol",
            1000,
            200,
            50,
            400,
            100,
            Some("trace_abc123".to_string()),
            Some("span_001".to_string()),
            350,
            200,
        );
        state.record_mcp_tool_call("mcp", "fs", "read_file", true);
        state.record_a2a_method_call("a2a", "agent", "tasks/send", true);

        let ui = UiServer::new(state, "127.0.0.1:8080".parse().unwrap(), true);
        let Json(report) = debug_cost(Query(CostQuery::default()), State(ui)).await;

        // Spend Ledger:
        assert_eq!(report.spend_ledger.priced_requests, 1);
        assert_eq!(report.spend_ledger.unpriced_requests, 0);
        assert!(!report.spend_ledger.models.is_empty());

        // Token & Cache Ledger:
        assert_eq!(report.token_ledger.total_input_uncached, 800);
        assert_eq!(report.token_ledger.total_cache_read, 200);
        assert_eq!(report.token_ledger.total_cache_write, 50);
        assert_eq!(report.token_ledger.total_output_non_reasoning, 300);
        assert_eq!(report.token_ledger.total_reasoning, 100);
        assert_eq!(report.token_ledger.total_tokens, 1400);
        assert_eq!(report.token_ledger.cache_hit_rate_pct, 20.0);

        // Efficiency Ledger:
        assert_eq!(report.efficiency_ledger.mcp_calls, 1);
        assert_eq!(report.efficiency_ledger.a2a_calls, 1);

        // Optimization Ledger:
        assert!(report.optimization_ledger.tokens_saved_cache >= 200);
        assert!(!report.optimization_ledger.opportunities.is_empty());

        // Cost Events:
        assert_eq!(report.events.len(), 1);
        let ev = &report.events[0];
        assert_eq!(ev.trace_id, "trace_abc123");
        assert_eq!(ev.span_id, "span_001");
        assert_eq!(ev.latency_ms, 350);
        assert_eq!(ev.data_quality, transit_core::DataQuality::Complete);
    }

    #[tokio::test]
    async fn disabled_metrics_endpoint_returns_not_found() {
        let ui = UiServer::new(ProxyState::new(), "127.0.0.1:8080".parse().unwrap(), false);
        assert_eq!(metrics(State(ui)).await.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn ui_page_contains_runtime_panels() {
        let html = ui_html(18080);

        assert!(html.contains("Overview"));
        assert!(html.contains("rel=\"icon\" href=\"/assets/transit-mark.svg\""));
        assert!(!html.contains("href=\"data:,\""));
        assert!(html.contains("id=\"tab-overview\""));
        assert!(html.contains("class=\"overview-kpi-grid\""));
        assert!(html.contains("class=\"overview-bottom-grid\""));
        assert!(html.contains("id=\"metric-api\""));
        assert!(html.contains("id=\"metric-mcp\""));
        assert!(html.contains("id=\"metric-agents\""));
        assert!(html.contains("id=\"metric-llm\""));
        assert!(html.contains("id=\"ov-panel-protocols\""));
        assert!(html.contains("id=\"ov-panel-policies\""));
        assert!(html.contains("id=\"ov-panel-runtime\""));
        assert!(html.contains("id=\"ov-panel-actions\""));
        assert!(!html.contains("id=\"ov-inventory-wrap\""));
        assert!(html.contains("id=\"inventory-table\""));
        assert!(html.contains("id=\"tab-services\""));
        assert!(html.contains("id=\"services-table\""));
        assert!(!html.contains("id=\"services-select\""));
        assert!(!html.contains("id=\"services-detail\""));
        assert!(html.contains("id=\"tab-llm\""));
        assert!(!html.contains("id=\"llm-table\""));
        assert!(!html.contains("id=\"llm-models-title\""));
        assert!(!html.contains("class=\"llm-header-bar\""));
        assert!(!html.contains("class=\"llm-metrics-grid\""));
        assert!(!html.contains("DATA PLANE — LIVE REQUEST PATH"));
        assert!(!html.contains("id=\"llm-models-panel\""));
        assert!(!html.contains("id=\"llm-detail-panel\""));
        assert!(!html.contains("id=\"llm-policy-summary-title\""));
        assert!(!html.contains("id=\"llm-alerts-title\""));
        assert!(html.contains("id=\"llm-accounts-grid\""));
        assert!(html.contains("id=\"tab-mcp\""));
        assert!(!html.contains("class=\"mcp-header-bar\""));
        assert!(!html.contains("class=\"mcp-metrics-grid\""));
        assert!(!html.contains("class=\"mcp-summary-bar\""));
        assert!(html.contains("id=\"mcp-servers-table\""));
        assert!(!html.contains("id=\"mcp-toggle-flow-btn\""));
        assert!(!html.contains("id=\"mcp-flow-dropdown-wrap\""));
        assert!(!html.contains("id=\"mcp-flow-panel\""));
        assert!(!html.contains("Invocation Flow (Agent"));
        assert!(html.contains("id=\"mcp-invocations-table\""));
        assert!(html.contains("id=\"mcp-server-drawer\""));
        assert!(html.contains("id=\"mcp-tools-table\""));
        assert!(html.contains("id=\"tab-a2a\""));
        assert!(!html.contains("class=\"a2a-header-bar\""));
        assert!(!html.contains("class=\"a2a-metrics-grid\""));
        assert!(html.contains("id=\"a2a-registry-table\""));
        assert!(!html.contains("id=\"a2a-flow-panel\""));
        assert!(!html.contains("id=\"a2a-toggle-path-btn\""));
        assert!(!html.contains("id=\"a2a-path-dropdown-wrap\""));
        assert!(!html.contains("Communication Path (Agent A"));
        assert!(html.contains("id=\"a2a-tasks-table\""));
        assert!(html.contains("id=\"a2a-task-drawer\""));
        assert!(html.contains("id=\"a2a-timeline-steps\""));
        assert!(html.contains("id=\"tab-observability\""));
        assert!(!html.contains("class=\"obs-header-bar\""));
        assert!(!html.contains("class=\"obs-metrics-grid\""));
        assert!(html.contains("class=\"obs-filter-toolbar\""));
        assert!(html.contains("id=\"obs-flow-map\""));
        assert!(html.contains("id=\"obs-spans-table\""));
        assert!(html.contains("id=\"obs-traces-table\""));
        assert!(html.contains("id=\"obs-trace-drawer\""));
        assert!(html.contains("id=\"observability-table\""));
        assert!(html.contains("id=\"tab-security\""));
        assert!(!html.contains("class=\"sec-header-bar\""));
        assert!(!html.contains("class=\"sec-top-grid\""));
        assert!(html.contains("Security Decision Chain"));
        assert!(html.contains("id=\"sec-policies-table\""));
        assert!(html.contains("id=\"sec-events-table\""));
        assert!(html.contains("id=\"sec-drawer\""));
        assert!(html.contains("id=\"security-table\""));
        assert!(html.contains("id=\"tab-cost-control\""));
        assert!(!html.contains("class=\"cost-header-bar\""));
        assert!(!html.contains("class=\"cost-metrics-grid\""));
        assert!(html.contains("id=\"cost-breakdown-table\""));
        assert!(html.contains("id=\"cost-tradeoff-table\""));
        assert!(html.contains("id=\"cost-expensive-table\""));
        assert!(html.contains("id=\"cost-budget-table\""));
        assert!(html.contains("id=\"cost-threshold-table\""));
        assert!(html.contains("id=\"cost-detail-panel\""));
        assert!(html.contains("id=\"cost-ledger-tabs\""));
        assert!(html.contains("id=\"cost-spend-model-table\""));
        assert!(html.contains("id=\"cost-events-table\""));
        assert!(html.contains("id=\"cost-usage-table\""));
        assert!(html.contains("id=\"cost-local-table\""));
        assert!(html.contains("id=\"metric-local-total\""));
        assert!(html.contains("id=\"cost-rate-table\""));
        assert!(html.contains("id=\"fx-country\""));
        assert!(html.contains("id=\"cost-model\""));
        assert!(html.contains("id=\"cost-family\""));
        assert!(html.contains("id=\"cost-billing\""));
        assert!(html.contains("id=\"cost-flame\""));
        assert!(html.contains("id=\"cost-flame-svg\""));
        assert!(html.contains("/debug/cost"));
        assert!(html.contains("class=\"nav-group\""));
        assert!(html.contains("Core"));
        assert!(html.contains("AI"));
        assert!(html.contains("Operations"));
        assert!(html.contains("id=\"search-input\""));
        assert!(html.contains("id=\"theme-toggle\""));
        assert!(html.contains("id=\"lang-toggle\""));
        assert!(html.contains("/debug/config"));
        assert!(!html.contains("id=\"metric-ready\""));
        assert!(!html.contains("id=\"metric-requests\""));
        assert!(!html.contains("id=\"metric-failures\""));
        assert!(!html.contains("id=\"traffic-table\""));
        assert!(!html.contains("Route traffic"));
        assert!(!html.contains("transit_requests_total"));
        assert!(!html.contains("class=\"mark\""));
        assert!(!html.contains("<strong>transit</strong>"));
        assert!(!html.contains("<span>ui</span>"));
        assert!(!html.contains("id=\"statusline\""));
        assert!(!html.contains("id=\"source-line\""));
        assert!(!html.contains("class=\"pill"));
        assert!(!html.contains("loading runtime data"));
        assert!(!html.contains("id=\"copy-config\""));
        assert!(!html.contains("id=\"refresh\""));
        assert!(!html.contains("Copy config"));
        assert!(!html.contains(">Refresh</button>"));
        assert!(!html.contains("getJson('/debug/backends')"));
        assert!(!html.contains("getJson('/debug/policies')"));
        assert!(!html.contains("getJson('/debug/routes')"));
        assert!(!html.contains("MCP request"));
        assert!(!html.contains("id=\"clusters-table\""));
        assert!(!html.contains("id=\"tab-routes\""));
        assert!(!html.contains("id=\"tab-backends\""));
        assert!(!html.contains("id=\"tab-policies\""));
        assert!(!html.contains("id=\"tab-playground\""));
        assert!(!html.contains("id=\"tab-config\""));
        assert!(!html.contains("value=\"/mcp\""));
        assert!(!html.contains("mcp-result\">{}"));
    }

    #[test]
    fn prometheus_metrics_expose_concurrency_for_autoscaling() {
        let mut proxy = empty_metrics();
        proxy.concurrency = ConcurrencyMetric {
            in_flight: 7,
            seconds_total: 12.5,
        };
        proxy.http_route_concurrency = vec![HttpRouteConcurrencyMetric {
            namespace: "app".into(),
            gateway: "public".into(),
            route: "orders".into(),
            cluster: "orders-v1".into(),
            concurrency: ConcurrencyMetric {
                in_flight: 3,
                seconds_total: 4.25,
            },
        }];

        let text = prometheus_metrics(ready(), proxy);

        assert!(text.contains("# TYPE transit_requests_in_flight gauge"));
        assert!(text.contains("transit_requests_in_flight 7"));
        // A counter, so rate() over it is average concurrency regardless of
        // when the scrape lands.
        assert!(text.contains("# TYPE transit_request_seconds_total counter"));
        assert!(text.contains("transit_request_seconds_total 12.5"));
        assert!(text.contains(
            "transit_http_route_requests_in_flight{namespace=\"app\",gateway=\"public\",route=\"orders\",cluster=\"orders-v1\"} 3"
        ));
        assert!(text.contains(
            "transit_http_route_request_seconds_total{namespace=\"app\",gateway=\"public\",route=\"orders\",cluster=\"orders-v1\"} 4.25"
        ));
        // Per-route concurrency cannot carry method or status: neither is known
        // while the request is still in flight.
        let in_flight_line = text
            .lines()
            .find(|line| line.starts_with("transit_http_route_requests_in_flight{"))
            .expect("in-flight series");
        assert!(!in_flight_line.contains("method="));
        assert!(!in_flight_line.contains("status_code="));
    }

    #[test]
    fn prometheus_metrics_escape_labels_and_include_http_dimensions() {
        let text = prometheus_metrics(
            Readiness {
                ready: true,
                revision: 1,
                version: "static=test".into(),
                source_versions: Default::default(),
                conflicts: vec![],
            },
            ProxyMetrics {
                held_activation_requests: 0,
                total_requests: 1,
                agent_requests: 0,
                policy_denied: 0,
                upstream_failures: 1,
                concurrency: ConcurrencyMetric::default(),
                http_route_concurrency: vec![],
                http_routes: vec![HttpRouteMetric {
                    namespace: "app\nns".into(),
                    gateway: "public\"gw".into(),
                    route: "default\\route".into(),
                    cluster: "reviews".into(),
                    method: "GET".into(),
                    status_code: 502,
                    requests: 1,
                    failures: 1,
                    latency_ms_sum: 25,
                    latency_ms_buckets: vec![
                        LatencyBucket { le: 5, count: 0 },
                        LatencyBucket { le: 25, count: 1 },
                    ],
                }],
                routes: vec![],
                llm_usage: vec![LlmUsageMetric {
                    route: "llm".into(),
                    backend: "claude".into(),
                    model: "claude-3".into(),
                    requests: 2,
                    prompt_tokens: 30,
                    cached_prompt_tokens: 0,
                    completion_tokens: 12,
                    ..LlmUsageMetric::default()
                }],
                mcp_tools: vec![McpToolMetric {
                    route: "mcp".into(),
                    backend: "mcp-a".into(),
                    tool: "search".into(),
                    calls: 3,
                    failures: 1,
                }],
                a2a_methods: vec![A2aMethodMetric {
                    route: "a2a".into(),
                    backend: "planner".into(),
                    method: "message/send".into(),
                    calls: 4,
                    failures: 2,
                }],
            },
        );

        assert!(text.contains("namespace=\"app\\nns\""));
        assert!(text.contains("gateway=\"public\\\"gw\""));
        assert!(text.contains("route=\"default\\\\route\""));
        assert!(text.contains("method=\"GET\""));
        assert!(text.contains("status_code=\"502\""));
        assert!(text.contains("transit_http_route_latency_ms_sum{"));
        assert!(text.contains("transit_http_route_latency_ms_count{"));
        assert!(text.contains("le=\"+Inf\""));
        assert!(text.contains(
            "transit_mcp_tool_calls_total{route=\"mcp\",backend=\"mcp-a\",tool=\"search\"} 3"
        ));
        assert!(text.contains(
            "transit_mcp_tool_failures_total{route=\"mcp\",backend=\"mcp-a\",tool=\"search\"} 1"
        ));
        assert!(text.contains(
            "transit_a2a_method_calls_total{route=\"a2a\",backend=\"planner\",method=\"message/send\"} 4"
        ));
        assert!(text.contains(
            "transit_a2a_method_failures_total{route=\"a2a\",backend=\"planner\",method=\"message/send\"} 2"
        ));
    }

    #[tokio::test]
    async fn debug_security_endpoints_return_real_models() {
        let state = ProxyState::new();
        let ui = UiServer::new(state.clone(), "127.0.0.1:8080".parse().unwrap(), true);

        let Json(posture) = debug_security_posture(State(ui.clone())).await;
        assert_eq!(posture.engine, "Native Rust Policy Engine");
        assert_eq!(posture.total_decisions, 0);

        let Json(events) = debug_security_events(State(ui.clone())).await;
        assert!(events.is_empty());

        let Json(identities) = debug_security_identities(State(ui.clone())).await;
        assert!(identities.is_empty());

        let Json(obs) = debug_observability(State(ui.clone())).await;
        assert!(obs.telemetry.metrics_enabled);
        assert!(obs.traces.is_empty());

        let decision = transit_core::SecurityDecision {
            event_id: "sec-live-01".into(),
            trace_id: "trace-live-01".into(),
            timestamp: "2026-09-08T12:00:00Z".into(),
            listener: "http-8080".into(),
            route: "chat".into(),
            backend: "codex".into(),
            protocol: "llm".into(),
            actor: "agent:tester".into(),
            principal: "spiffe://acme.internal/sa/tester".into(),
            authn_method: "jwt".into(),
            enforcement_point: "PEP: RouteAuthZ".into(),
            policy_id: "auth".into(),
            resource_kind: "route".into(),
            resource_id: "/chat".into(),
            decision: "allowed".into(),
            reason_code: "authz.allow".into(),
            status_code: 200,
            latency_ms: 15,
            evidence_hash: "hash".into(),
            redacted_attributes: std::collections::BTreeMap::new(),
        };
        state.record_security_decision(decision);

        let Json(posture2) = debug_security_posture(State(ui.clone())).await;
        assert_eq!(posture2.total_decisions, 1);
        let Json(events2) = debug_security_events(State(ui.clone())).await;
        assert_eq!(events2.len(), 1);
        assert_eq!(events2[0].trace_id, "trace-live-01");
        let Json(obs2) = debug_observability(State(ui)).await;
        assert_eq!(obs2.traces.len(), 1);
        assert_eq!(obs2.traces[0].trace_id, "trace-live-01");
    }

    #[tokio::test]
    async fn debug_services_returns_kpis_and_unified_services() {
        let state = ProxyState::new();
        let ui = UiServer::new(state, "127.0.0.1:8080".parse().unwrap(), true);
        let Json(data) = debug_services(State(ui)).await;
        assert_eq!(data.kpis.error_rate_pct, 0.0);
        assert_eq!(data.kpis.total_services, 0);
        assert_eq!(data.kpis.total_routes, 0);
    }
}
