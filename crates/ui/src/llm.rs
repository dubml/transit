use super::UiServer;
use axum::extract::{ConnectInfo, DefaultBodyLimit, OriginalUri, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use transit_core::{BackendKind, ProviderKind};
use transit_proxy::OAuthAccount;

type ApiResult = Result<Response, (StatusCode, Json<Value>)>;

pub(super) fn routes() -> Router<UiServer> {
    Router::new()
        .route("/assets/llm.js", get(script))
        .route("/assets/llm.css", get(styles))
        .route("/debug/llm", get(dashboard))
        .route("/admin/llm/session", post(local_session))
        .route("/admin/llm/oauth/start", post(start_login))
        .route(
            "/admin/llm/oauth/:id",
            get(login_status).delete(cancel_login),
        )
        .route("/admin/llm/oauth/:id/callback", post(finish_login))
        .route(
            "/admin/llm/accounts/:id",
            get(read_account).put(save_account).delete(delete_account),
        )
        .route("/admin/llm/accounts/:id/download", get(download_account))
        .route("/admin/llm/accounts/:id/refresh", post(refresh_account))
        .route(
            "/admin/llm/accounts/:id/status",
            axum::routing::patch(account_status),
        )
        .route("/admin/llm/accounts/:id/quota", post(account_quota))
        .route("/admin/llm/accounts/:id/models", get(account_models))
        .route(
            "/admin/llm/accounts/:id/reset-quota",
            post(reset_account_quota),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
}

async fn script() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../../../ui/llm.js"),
    )
}

async fn styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../../../ui/llm.css"),
    )
}

fn error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"error":message.into()})))
}

fn authorize(ui: &UiServer, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    if ui.state.access_settings().configured() {
        return if ui
            .state
            .access_settings()
            .authorized(super::access::token(headers))
        {
            Ok(())
        } else {
            Err(error(StatusCode::UNAUTHORIZED, "Management key required"))
        };
    }
    if !ui.state.llm_accounts().enabled() {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Configure TRANSIT_LLM_ACCOUNTS_DIR and TRANSIT_LLM_ADMIN_TOKEN to manage OAuth accounts",
        ));
    }
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if !ui.state.llm_accounts().authorized(token) {
        return Err(error(
            StatusCode::UNAUTHORIZED,
            "LLM management authentication required",
        ));
    }
    Ok(())
}

fn private_json(value: impl serde::Serialize) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}

async fn local_session(
    State(ui): State<UiServer>,
    peer: Option<ConnectInfo<std::net::SocketAddr>>,
    headers: HeaderMap,
) -> ApiResult {
    if ui.state.access_settings().configured() {
        return Ok(super::access::local_session(State(ui), peer, headers).await);
    }
    let addr = ui.bind_addr.filter(|addr| addr.ip().is_loopback());
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let trusted = addr.is_some_and(|addr| {
        (host == addr.to_string() || host == format!("localhost:{}", addr.port()))
            && origin == format!("http://{host}")
    }) && peer.is_some_and(|peer| peer.0.ip().is_loopback());
    if !trusted {
        return Err(error(
            StatusCode::FORBIDDEN,
            "Local management sessions require a same-origin loopback connection",
        ));
    }
    let token = ui
        .state
        .llm_accounts()
        .local_session_token()
        .ok_or_else(|| {
            error(
                StatusCode::FORBIDDEN,
                "This gateway requires an explicit management token",
            )
        })?;
    Ok(private_json(json!({"token":token})))
}

async fn read_account(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let account = ui
        .state
        .llm_accounts()
        .get(&id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Account not found"))?;
    Ok(private_json(account))
}

async fn save_account(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(account): Json<OAuthAccount>,
) -> ApiResult {
    authorize(&ui, &headers)?;
    if id != account.id {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Account ID does not match URL",
        ));
    }
    validate_binding(&ui.state, &account.backend, account.provider())?;
    let result = ui.state.llm_accounts().save(account).await.map_err(|e| {
        let status = if e.contains("Account changed") {
            StatusCode::CONFLICT
        } else {
            StatusCode::BAD_REQUEST
        };
        error(status, e)
    })?;
    Ok(private_json(result))
}

fn validate_binding(
    state: &transit_proxy::ProxyState,
    name: &str,
    oauth_provider: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    if !matches!(oauth_provider, "codex" | "claude") {
        return Err(error(StatusCode::BAD_REQUEST, "Unsupported OAuth provider"));
    }
    if name.is_empty() {
        return Ok(());
    }
    let snapshot = state.snapshot();
    let backend = snapshot
        .backend(name)
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "LLM backend does not exist"))?;
    if !matches!(backend.kind, BackendKind::Llm { .. }) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Account must bind to an LLM backend",
        ));
    }
    if let BackendKind::Llm {
        account_type,
        provider,
        ..
    } = &backend.kind
    {
        if !matches!(account_type.as_deref(), Some("subscription" | "oauth")) {
            return Err(error(
                StatusCode::BAD_REQUEST,
                "OAuth accounts require a subscription backend",
            ));
        }
        let configured = snapshot.provider(provider).map(|provider| provider.kind);
        let compatible = match oauth_provider {
            "claude" => configured == Some(ProviderKind::Anthropic),
            "codex" => matches!(
                configured,
                Some(ProviderKind::OpenAi | ProviderKind::OpenAiCompatible)
            ),
            _ => false,
        };
        if !compatible {
            return Err(error(
                StatusCode::BAD_REQUEST,
                "OAuth provider does not match the backend provider",
            ));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginRequest {
    provider: String,
    #[serde(default)]
    backend: String,
}

async fn start_login(
    State(ui): State<UiServer>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> ApiResult {
    authorize(&ui, &headers)?;
    validate_binding(&ui.state, &request.backend, &request.provider)?;
    let login = ui
        .state
        .llm_accounts()
        .start_login(&request.provider, &request.backend)
        .map_err(|e| error(StatusCode::BAD_REQUEST, e))?;
    let automatic = ui
        .oauth_callbacks
        .ensure(&request.provider, ui.state.clone());
    let mut value = serde_json::to_value(login)
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "Cannot serialize login"))?;
    value["callback_mode"] = json!(if automatic { "automatic" } else { "manual" });
    Ok(private_json(value))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CallbackRequest {
    callback_url: String,
}

async fn finish_login(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CallbackRequest>,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let (provider, backend) = ui
        .state
        .llm_accounts()
        .login_backend(&id)
        .map_err(|e| error(StatusCode::BAD_REQUEST, e))?;
    validate_binding(&ui.state, &backend, &provider)?;
    let account = ui
        .state
        .llm_accounts()
        .finish_login(&id, &request.callback_url)
        .await
        .map_err(|e| error(StatusCode::BAD_REQUEST, e))?;
    Ok(private_json(account))
}

async fn login_status(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    Ok(private_json(ui.state.llm_accounts().login_status(&id)))
}

async fn cancel_login(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    ui.state.llm_accounts().cancel_login(&id).await;
    Ok(private_json(json!({"status":"cancelled"})))
}

#[derive(Default)]
pub(super) struct CallbackListeners(
    std::sync::Mutex<std::collections::HashMap<String, tokio::task::JoinHandle<()>>>,
);

impl Drop for CallbackListeners {
    fn drop(&mut self) {
        for (_, task) in self.0.get_mut().unwrap().drain() {
            task.abort();
        }
    }
}

#[derive(Clone)]
struct CallbackContext {
    state: transit_proxy::ProxyState,
    provider: String,
    redirect: String,
}

impl CallbackListeners {
    fn ensure(&self, provider: &str, state: transit_proxy::ProxyState) -> bool {
        let mut listeners = self.0.lock().unwrap();
        if listeners
            .get(provider)
            .is_some_and(|task| !task.is_finished())
        {
            return true;
        }
        let (port, path) = match provider {
            "codex" => (1455, "/auth/callback"),
            "claude" => (54545, "/callback"),
            _ => return false,
        };
        let Ok(listener) = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        else {
            return false;
        };
        if listener.set_nonblocking(true).is_err() {
            return false;
        }
        let Ok(server) = axum::Server::from_tcp(listener) else {
            return false;
        };
        let context = CallbackContext {
            state,
            provider: provider.into(),
            redirect: format!("http://localhost:{port}{path}"),
        };
        let app = Router::new()
            .route(path, get(browser_callback))
            .with_state(context);
        listeners.insert(
            provider.into(),
            tokio::spawn(async move {
                let _ = server.serve(app.into_make_service()).await;
            }),
        );
        true
    }
}

async fn browser_callback(
    State(context): State<CallbackContext>,
    OriginalUri(uri): OriginalUri,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let state = query.get("state").map(String::as_str).unwrap_or("");
    let result = async {
        let id = context
            .state
            .llm_accounts()
            .login_id_for_state(&context.provider, state)
            .ok_or("Unknown or expired login")?;
        let (provider, backend) = context.state.llm_accounts().login_backend(&id)?;
        validate_binding(&context.state, &backend, &provider)
            .map_err(|_| "Login backend changed; start a new login")?;
        let callback = format!("{}?{}", context.redirect, uri.query().unwrap_or(""));
        context
            .state
            .llm_accounts()
            .finish_login(&id, &callback)
            .await?;
        Ok::<_, String>(())
    }
    .await;
    let (status, message) = if result.is_ok() {
        (
            StatusCode::OK,
            "Sign-in complete. Return to the transit window. 登录成功，请返回 transit 页面。",
        )
    } else {
        (StatusCode::BAD_REQUEST, "Sign-in could not complete. Return to transit to check the status or paste the callback URL. 登录未完成，请返回 transit 检查状态或提交回调 URL。")
    };
    (
        status,
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::REFERRER_POLICY, "no-referrer"),
        ],
        format!("<!doctype html><meta charset=utf-8><title>transit OAuth</title><p>{message}</p>"),
    )
        .into_response()
}

async fn download_account(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let account = ui
        .state
        .llm_accounts()
        .get(&id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "Account not found"))?;
    let mut response = private_json(account.document);
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{id}.json\"")
            .parse()
            .unwrap(),
    );
    Ok(response)
}

async fn refresh_account(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    if ui.state.llm_accounts().get(&id).is_none() {
        return Err(error(StatusCode::NOT_FOUND, "Account not found"));
    }
    let summary = ui
        .state
        .llm_accounts()
        .refresh(&id)
        .await
        .map_err(|e| error(StatusCode::BAD_GATEWAY, e))?;
    Ok(private_json(summary))
}

#[derive(Deserialize)]
struct AccountRevision {
    revision: u64,
}
#[derive(Deserialize)]
struct AccountStatus {
    revision: u64,
    disabled: bool,
}
#[derive(Deserialize)]
struct QuotaReset {
    redeem_request_id: String,
    credit_id: String,
}

fn management_error(message: String) -> (StatusCode, Json<Value>) {
    let code = if message == "Account not found" {
        StatusCode::NOT_FOUND
    } else if message.starts_with("Account changed") || message.starts_with("Reset conflict:") {
        StatusCode::CONFLICT
    } else if message.starts_with("Provider") || message.starts_with("Invalid provider") {
        StatusCode::BAD_GATEWAY
    } else {
        StatusCode::BAD_REQUEST
    };
    error(code, message)
}

async fn delete_account(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AccountRevision>,
) -> ApiResult {
    authorize(&ui, &headers)?;
    ui.state
        .llm_accounts()
        .delete(&id, request.revision)
        .await
        .map_err(management_error)?;
    Ok(private_json(json!({"deleted": id})))
}

async fn account_status(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AccountStatus>,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let result = ui
        .state
        .llm_accounts()
        .set_disabled(&id, request.revision, request.disabled)
        .await
        .map_err(management_error)?;
    Ok(private_json(result))
}

async fn account_quota(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let result = ui
        .state
        .llm_accounts()
        .account_quota(&id)
        .await
        .map_err(management_error)?;
    Ok(private_json(result))
}

async fn reset_account_quota(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<QuotaReset>,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let result = ui
        .state
        .llm_accounts()
        .reset_account_quota(&id, &request.redeem_request_id, &request.credit_id)
        .await
        .map_err(management_error)?;
    Ok(private_json(result))
}

async fn account_models(
    State(ui): State<UiServer>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> ApiResult {
    authorize(&ui, &headers)?;
    let models = ui
        .state
        .llm_accounts()
        .account_models(&id)
        .await
        .map_err(management_error)?;
    let revision = ui
        .state
        .llm_accounts()
        .get(&id)
        .ok_or_else(|| management_error("Account not found".into()))?
        .revision;
    Ok(private_json(json!({"models":models,"revision":revision})))
}

async fn dashboard(State(ui): State<UiServer>) -> Response {
    let config = ui.state.snapshot().to_redacted_runtime_config();
    let metrics = ui.state.metrics();
    let accounts = ui.state.llm_accounts().list();
    let backends: Vec<Value> = config.backends.iter().filter_map(|backend| {
        let BackendKind::Llm { provider, models, account_type, quota_state, .. } = &backend.kind else { return None; };
        let provider_config = config.providers.iter().find(|p| p.name == *provider);
        let bound = accounts.iter().find(|a| a.backend == backend.name);
        let mode = match (bound.is_some(), account_type.as_deref()) {
            (true, _) | (_, Some("subscription" | "oauth")) => "subscription",
            (_, Some("self-hosted" | "local" | "open-source")) => "local",
            _ => "api",
        };
        let family = match bound.map(|a| a.provider.as_str()) {
            Some("claude") => "anthropic",
            Some("codex") => "chatgpt",
            _ if provider_config.is_some_and(|p| p.kind == ProviderKind::Anthropic) => "anthropic",
            _ => "chatgpt",
        };
        let usage: Vec<_> = metrics.llm_usage.iter().filter(|u| u.backend == backend.name).collect();
        let sum = |f: fn(&transit_proxy::LlmUsageMetric) -> u64| usage.iter().map(|u| f(u)).sum::<u64>();
        let observed_requests = sum(|u| u.requests);
        let priced_requests = sum(|u| u.priced_requests);
        let input = sum(|u| u.prompt_tokens);
        let output = sum(|u| u.completion_tokens);
        let spend = usage.iter().map(|u| u.estimated_usd_nanos).sum::<u128>();
        let timing = ui.state.llm_timings().summary(&backend.name);
        Some(json!({
            "name":backend.name,"provider":provider,"family":family,"mode":mode,"models":models,
            "endpoint":backend.endpoint(provider_config),"quota":quota_state,
            "requests":metrics.routes.iter().filter(|r| r.backend == backend.name && r.protocol == "llm").map(|r| r.requests).sum::<u64>(),
            "usage_requests":observed_requests,"input_tokens":input,"output_tokens":output,"total_tokens":input+output,
            "cached_input_tokens":sum(|u|u.cached_prompt_tokens),"cache_write_tokens":sum(|u|u.cache_write_tokens),
            "reasoning_tokens":if family == "anthropic" { None } else { Some(sum(|u|u.reasoning_tokens)) },"priced_requests":priced_requests,
            "estimated_usd":if priced_requests>0 {Some(spend as f64/1_000_000_000.0)} else {None},
            "ttft_ms":timing.ttft_ms,"ttft_samples":timing.ttft_samples,"tokens_per_second":timing.tokens_per_second,
            "generated_tokens":timing.generated_tokens,"generation_seconds":timing.generation_seconds,
            "requests_per_second":timing.requests_per_second,"context":timing.context
        }))
    }).collect();
    private_json(
        json!({"accounts":accounts,"backends":backends,"management_enabled":ui.state.llm_accounts().enabled(),"management_mode":if if ui.state.access_settings().configured() {ui.state.access_settings().local_token().is_some()} else {ui.state.llm_accounts().local_session_token().is_some()} {"local"} else {"token"},"pricing_source":"published rate card; estimate, not provider invoice"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn local_session_checks_peer_host_origin_and_callback_status() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            Directory(std::env::temp_dir().join(format!("transit-local-session-{nonce}")));
        let state = transit_proxy::ProxyState::new();
        state
            .apply_config(
                serde_json::from_str(include_str!("../../../tests/ui-fake.json")).unwrap(),
            )
            .unwrap();
        state.llm_accounts().configure_local(&directory.0).unwrap();
        let mut ui = UiServer::new(state.clone(), "127.0.0.1:8080".parse().unwrap(), true);
        ui.bind_addr = Some("127.0.0.1:15021".parse().unwrap());
        let app = routes().with_state(ui.clone());
        for (peer, host, origin, expected) in [
            (
                "127.0.0.1:12345",
                "127.0.0.1:15021",
                "http://127.0.0.1:15021",
                StatusCode::OK,
            ),
            (
                "127.0.0.1:12345",
                "localhost:15021",
                "http://localhost:15021",
                StatusCode::OK,
            ),
            (
                "192.0.2.1:12345",
                "127.0.0.1:15021",
                "http://127.0.0.1:15021",
                StatusCode::FORBIDDEN,
            ),
            (
                "127.0.0.1:12345",
                "attacker.test:15021",
                "http://attacker.test:15021",
                StatusCode::FORBIDDEN,
            ),
            (
                "127.0.0.1:12345",
                "127.0.0.1:15021",
                "https://attacker.test",
                StatusCode::FORBIDDEN,
            ),
            (
                "127.0.0.1:12345",
                "127.0.0.1:15021",
                "",
                StatusCode::FORBIDDEN,
            ),
        ] {
            let mut request = axum::http::Request::builder()
                .method("POST")
                .uri("/admin/llm/session")
                .header(header::HOST, host)
                .header(header::ORIGIN, origin)
                .body(hyper::Body::empty())
                .unwrap();
            request
                .extensions_mut()
                .insert(ConnectInfo(peer.parse::<std::net::SocketAddr>().unwrap()));
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), expected);
            if expected == StatusCode::OK {
                assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
                let value: Value = serde_json::from_slice(
                    &hyper::body::to_bytes(response.into_body()).await.unwrap(),
                )
                .unwrap();
                assert!(state
                    .llm_accounts()
                    .authorized(value["token"].as_str().unwrap()));
            }
        }
        let login = state
            .llm_accounts()
            .start_login("codex", "codex-team-sub")
            .unwrap();
        let state_value = login
            .authorization_url
            .split("state=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap();
        let callbacks = Router::new()
            .route("/auth/callback", get(browser_callback))
            .with_state(CallbackContext {
                state: state.clone(),
                provider: "codex".into(),
                redirect: login.redirect_uri.clone(),
            });
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(callbacks.into_make_service()),
        );
        let response = hyper::Client::new()
            .get(
                format!("http://{addr}/auth/callback?state={state_value}&error=access_denied")
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
        server.abort();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            state.llm_accounts().login_status(&login.id)["status"],
            "error"
        );
        assert!(state.llm_accounts().login_status(&login.id)["error"]
            .as_str()
            .unwrap()
            .contains("denied"));
        ui.bind_addr = Some("0.0.0.0:15021".parse().unwrap());
        let mut request = axum::http::Request::builder()
            .method("POST")
            .uri("/admin/llm/session")
            .header(header::HOST, "localhost:15021")
            .header(header::ORIGIN, "http://localhost:15021")
            .body(hyper::Body::empty())
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(
            "127.0.0.1:12345".parse::<std::net::SocketAddr>().unwrap(),
        ));
        assert_eq!(
            routes()
                .with_state(ui)
                .oneshot(request)
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }

    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    async fn call(
        app: &Router,
        method: &str,
        path: &str,
        authenticated: bool,
        body: Value,
    ) -> Response {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json");
        if authenticated {
            request = request.header("authorization", "Bearer test-management-token-long-enough");
        }
        app.clone()
            .oneshot(request.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn account_controls_require_authentication_and_reject_stale_revisions() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            Directory(std::env::temp_dir().join(format!("transit-account-controls-{nonce}")));
        let state = transit_proxy::ProxyState::new();
        state
            .llm_accounts()
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        state
            .llm_accounts()
            .save(OAuthAccount {
                id: "controls".into(),
                backend: String::new(),
                document: json!({"type":"codex","access_token":"do-not-expose"}),
                models: vec![],
                revision: 0,
            })
            .await
            .unwrap();
        let app = routes().with_state(UiServer::new(
            state.clone(),
            "127.0.0.1:8080".parse().unwrap(),
            true,
        ));
        for (method, path, body) in [
            (
                "PATCH",
                "/admin/llm/accounts/controls/status",
                json!({"revision":1,"disabled":true}),
            ),
            (
                "DELETE",
                "/admin/llm/accounts/controls",
                json!({"revision":1}),
            ),
            ("POST", "/admin/llm/accounts/controls/quota", json!({})),
            ("GET", "/admin/llm/accounts/controls/models", json!({})),
            (
                "POST",
                "/admin/llm/accounts/controls/reset-quota",
                json!({"redeem_request_id":"00000000-0000-4000-8000-000000000000","credit_id":"credit-1"}),
            ),
        ] {
            assert_eq!(
                call(&app, method, path, false, body).await.status(),
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(
            call(
                &app,
                "PATCH",
                "/admin/llm/accounts/controls/status",
                true,
                json!({"revision":0,"disabled":true})
            )
            .await
            .status(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            call(
                &app,
                "PATCH",
                "/admin/llm/accounts/controls/status",
                true,
                json!({"revision":1,"disabled":true})
            )
            .await
            .status(),
            StatusCode::OK
        );
        assert!(state.llm_accounts().get("controls").unwrap().disabled());
        assert_eq!(
            call(
                &app,
                "DELETE",
                "/admin/llm/accounts/controls",
                true,
                json!({"revision":1})
            )
            .await
            .status(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            call(
                &app,
                "DELETE",
                "/admin/llm/accounts/controls",
                true,
                json!({"revision":2})
            )
            .await
            .status(),
            StatusCode::OK
        );
        assert_eq!(
            call(&app, "GET", "/admin/llm/accounts/controls", true, json!({}))
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
        assert!(!directory.0.join("controls.json").exists());
    }

    #[tokio::test]
    async fn empty_config_accepts_oauth_logins_and_unbound_accounts_for_both_providers() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            Directory(std::env::temp_dir().join(format!("transit-unbound-oauth-{nonce}")));
        let state = transit_proxy::ProxyState::new();
        state
            .llm_accounts()
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        let app = routes().with_state(UiServer::new(
            state.clone(),
            "127.0.0.1:8080".parse().unwrap(),
            true,
        ));
        for provider in ["codex", "claude"] {
            let response = call(
                &app,
                "POST",
                "/admin/llm/oauth/start",
                true,
                json!({"provider":provider}),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            let login: Value =
                serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await.unwrap())
                    .unwrap();
            assert!(login["authorization_url"]
                .as_str()
                .unwrap()
                .contains("code_challenge="));
            let id = login["id"].as_str().unwrap();
            assert_eq!(state.llm_accounts().login_backend(id).unwrap().1, "");
            state.llm_accounts().cancel_login(id).await;
            let account = json!({"id":provider,"document":{"type":provider,"access_token":"private-access","refresh_token":"private-refresh"}});
            assert_eq!(
                call(
                    &app,
                    "PUT",
                    &format!("/admin/llm/accounts/{provider}"),
                    true,
                    account
                )
                .await
                .status(),
                StatusCode::OK
            );
        }
        let response = call(&app, "GET", "/debug/llm", false, Value::Null).await;
        let raw = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let dashboard: Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(dashboard["backends"].as_array().unwrap().len(), 0);
        assert_eq!(dashboard["accounts"].as_array().unwrap().len(), 2);
        assert!(!String::from_utf8(raw.to_vec())
            .unwrap()
            .contains("private-access"));
        let restarted = transit_proxy::LlmAccounts::default();
        restarted
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        assert_eq!(restarted.list().len(), 2);
        assert!(restarted.get("codex").unwrap().backend.is_empty());
        assert!(restarted.get("claude").unwrap().backend.is_empty());
        assert_eq!(
            call(
                &app,
                "POST",
                "/admin/llm/oauth/start",
                true,
                json!({"provider":"codex","backend":"missing"})
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn account_http_api_authenticates_edits_exports_and_redacts_dashboard() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = Directory(
            std::env::temp_dir().join(format!("transit-admin-api-{}-{nonce}", std::process::id())),
        );
        let state = transit_proxy::ProxyState::new();
        let config = serde_json::from_str(include_str!("../../../tests/ui-fake.json")).unwrap();
        state.apply_config(config).unwrap();
        state
            .llm_accounts()
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        let app = routes().with_state(UiServer::new(
            state,
            "127.0.0.1:8080".parse().unwrap(),
            true,
        ));
        let account = json!({"id":"test","backend":"codex-team-sub","document":{"type":"codex","access_token":"private-access","refresh_token":"private-refresh","extension":true},"revision":0,"models":[{"model":"gpt-5","alias":"friendly","disabled":false,"reasoning_effort":"high"}]});
        let start = "/admin/llm/oauth/start";
        let request = json!({"provider":"codex","backend":"codex-team-sub"});
        assert_eq!(
            call(&app, "POST", start, false, request.clone())
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(
                &app,
                "POST",
                start,
                true,
                json!({"provider":"claude","backend":"codex-team-sub"})
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(
                &app,
                "POST",
                start,
                true,
                json!({"provider":"codex","backend":"missing"})
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
        let login = call(&app, "POST", start, true, request).await;
        assert_eq!(login.status(), StatusCode::OK);
        assert_eq!(login.headers()[header::CACHE_CONTROL], "no-store");
        let login: Value =
            serde_json::from_slice(&hyper::body::to_bytes(login.into_body()).await.unwrap())
                .unwrap();
        assert!(login["authorization_url"]
            .as_str()
            .unwrap()
            .starts_with("https://auth.openai.com/oauth/authorize?"));
        assert!(login.get("verifier").is_none());
        let callback = format!(
            "/admin/llm/oauth/{}/callback",
            login["id"].as_str().unwrap()
        );
        let payload =
            json!({"callback_url":"http://localhost:1455/auth/callback?code=x&state=wrong"});
        assert_eq!(
            call(&app, "POST", &callback, false, payload.clone())
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(&app, "POST", &callback, true, payload).await.status(),
            StatusCode::BAD_REQUEST
        );
        let path = "/admin/llm/accounts/test";
        assert_eq!(
            call(&app, "PUT", path, false, account.clone())
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let response = call(&app, "PUT", path, true, account.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            call(&app, "GET", path, false, Value::Null).await.status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(&app, "PUT", path, true, account.clone())
                .await
                .status(),
            StatusCode::CONFLICT
        );
        let download = call(
            &app,
            "GET",
            "/admin/llm/accounts/test/download",
            true,
            Value::Null,
        )
        .await;
        assert_eq!(
            download.headers()[header::CONTENT_DISPOSITION],
            "attachment; filename=\"test.json\""
        );
        let exported: Value =
            serde_json::from_slice(&hyper::body::to_bytes(download.into_body()).await.unwrap())
                .unwrap();
        assert_eq!(exported, account["document"]);
        let response = call(&app, "GET", "/debug/llm", false, Value::Null).await;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("private-"));
        let dashboard: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(dashboard["accounts"][0]["models"][0]["alias"], "friendly");
        assert_eq!(dashboard["accounts"][0]["revision"], 1);
        let mut invalid = account;
        invalid["revision"] = json!(1);
        invalid["backend"] = json!("not-configured");
        assert_eq!(
            call(&app, "PUT", path, true, invalid).await.status(),
            StatusCode::BAD_REQUEST
        );
    }
}
