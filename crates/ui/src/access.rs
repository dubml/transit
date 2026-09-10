use super::UiServer;
use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, State},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Mutex,
    time::{Duration, Instant},
};
use transit_proxy::access_settings::{bearer, AccessConfig, SaveError};

pub fn routes() -> Router<UiServer> {
    Router::new()
        .route("/admin/access/status", get(status))
        .route("/admin/session", post(local_session))
        .route("/admin/access", get(read).put(save))
        .route(
            "/assets/configuration.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../../../ui/configuration.js"),
                )
            }),
        )
        .route(
            "/assets/configuration.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../../../ui/configuration.css"),
                )
            }),
        )
        .layer(DefaultBodyLimit::max(512 * 1024))
}

fn response(code: StatusCode, value: Value) -> Response {
    (code, [(header::CACHE_CONTROL, "no-store")], Json(value)).into_response()
}

pub fn token(headers: &HeaderMap) -> &str {
    headers
        .get(header::AUTHORIZATION)
        .or_else(|| headers.get("x-management-key"))
        .and_then(|v| v.to_str().ok())
        .map(bearer)
        .unwrap_or("")
}

#[derive(Default)]
pub struct Attempts(Mutex<HashMap<std::net::IpAddr, (u8, Instant)>>);

impl Attempts {
    fn blocked(&self, ip: std::net::IpAddr) -> bool {
        let mut attempts = self.0.lock().unwrap();
        attempts.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(1800));
        attempts.get(&ip).is_some_and(|(count, _)| *count >= 5)
    }
    fn failed(&self, ip: std::net::IpAddr) {
        let mut attempts = self.0.lock().unwrap();
        // Keep the map bounded even when many distinct peers attempt authentication.
        if attempts.len() >= 4096 && !attempts.contains_key(&ip) {
            return;
        }
        let entry = attempts.entry(ip).or_insert((0, Instant::now()));
        entry.0 = entry.0.saturating_add(1);
        entry.1 = Instant::now();
    }
    fn clear(&self, ip: std::net::IpAddr) {
        self.0.lock().unwrap().remove(&ip);
    }
}

pub async fn guard<B>(State(ui): State<UiServer>, mut req: Request<B>, next: Next<B>) -> Response {
    let settings = ui.state.access_settings();
    // HTTP/2 carries :authority instead of Host; use it for the same-origin check.
    if !req.headers().contains_key(header::HOST) {
        if let Some(authority) = req
            .uri()
            .authority()
            .and_then(|value| value.as_str().parse().ok())
        {
            req.headers_mut().insert(header::HOST, authority);
        }
    }
    let Some(remote) = settings.remote() else {
        return next.run(req).await;
    };
    let path = req.uri().path();
    let panel = matches!(path, "/" | "/ui") || path.starts_with("/assets/");
    if panel && remote.disable_control_panel {
        return response(
            StatusCode::NOT_FOUND,
            json!({"error":"Control panel disabled"}),
        );
    }
    let management = path.starts_with("/admin/") || path.starts_with("/debug/");
    if !management {
        return next.run(req).await;
    }
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|p| p.0);
    let Some(peer) = peer else {
        return response(
            StatusCode::FORBIDDEN,
            json!({"error":"Management peer address unavailable"}),
        );
    };
    if !peer.ip().is_loopback() && !remote.allow_remote {
        return response(
            StatusCode::FORBIDDEN,
            json!({"error":"Remote management disabled"}),
        );
    }
    if matches!(
        path,
        "/admin/session" | "/admin/llm/session" | "/admin/access/status"
    ) {
        return next.run(req).await;
    }
    if let Some(origin) = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
    {
        let host = req
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let scheme = if settings.active_tls().enable {
            "https"
        } else {
            "http"
        };
        if origin != format!("{scheme}://{host}") {
            return response(
                StatusCode::FORBIDDEN,
                json!({"error":"Cross-origin management request rejected"}),
            );
        }
    }
    if ui.management_attempts.blocked(peer.ip()) {
        return response(
            StatusCode::TOO_MANY_REQUESTS,
            json!({"error":"Too many invalid management keys. Retry after 30 minutes."}),
        );
    }
    let candidate = token(req.headers()).to_string();
    let checked = candidate.clone();
    let state = ui.state.clone();
    let authorized =
        tokio::task::spawn_blocking(move || state.access_settings().authorized(&checked))
            .await
            .unwrap_or(false);
    if !authorized {
        if !candidate.is_empty() {
            ui.management_attempts.failed(peer.ip());
        }
        return response(
            StatusCode::UNAUTHORIZED,
            json!({"error":"Management key required"}),
        );
    }
    ui.management_attempts.clear(peer.ip());
    next.run(req).await
}

async fn status(State(ui): State<UiServer>) -> Response {
    response(
        StatusCode::OK,
        json!({"enabled":ui.state.access_settings().configured(),"management_mode":if ui.state.access_settings().local_token().is_some(){"local"}else{"token"},"name":ui.build.name,"version":ui.build.version,"runtime":ui.runtime}),
    )
}

pub async fn local_session(
    State(ui): State<UiServer>,
    peer: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
) -> Response {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let scheme = if ui.state.access_settings().active_tls().enable {
        "https"
    } else {
        "http"
    };
    let trusted_host = ui.bind_addr.is_some_and(|addr| {
        host.parse::<axum::http::uri::Authority>()
            .is_ok_and(|authority| {
                let name = authority
                    .host()
                    .trim_start_matches('[')
                    .trim_end_matches(']');
                let local = name.eq_ignore_ascii_case("localhost")
                    || name
                        .parse::<std::net::IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback());
                local
                    && authority
                        .port_u16()
                        .unwrap_or(if scheme == "https" { 443 } else { 80 })
                        == addr.port()
            })
    });
    if !trusted_host
        || origin != format!("{scheme}://{host}")
        || !peer.is_some_and(|peer| peer.0.ip().is_loopback())
    {
        return response(
            StatusCode::FORBIDDEN,
            json!({"error":"Local management sessions require a same-origin loopback connection"}),
        );
    }
    match ui.state.access_settings().local_token() {
        Some(token) => response(StatusCode::OK, json!({"token":token})),
        None => response(
            StatusCode::FORBIDDEN,
            json!({"error":"This gateway requires an explicit management key"}),
        ),
    }
}

async fn read(State(ui): State<UiServer>) -> Response {
    match ui.state.access_settings().view() {
        Some(view) => response(
            StatusCode::OK,
            json!({"settings":view,"build":ui.build,"runtime":ui.runtime,"management_address":ui.bind_addr,"panel":ui.panel_assets.status(&ui.state).await}),
        ),
        None => response(
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"error":"Access configuration is unavailable"}),
        ),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveRequest {
    revision: u64,
    config: AccessConfig,
    secret: Option<String>,
}

async fn save(State(ui): State<UiServer>, Json(request): Json<SaveRequest>) -> Response {
    if ui.runtime.mode == transit_core::RuntimeMode::Kubernetes {
        return response(StatusCode::CONFLICT, json!({
            "error":"Access settings are managed by the Kubernetes deployment; update its ConfigMap or Secret and roll out the change.",
            "managed_by":"kubernetes"
        }));
    }
    let host = match request.config.host.trim() {
        "" => Some(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
        "localhost" => Some(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
        value => value.parse::<std::net::IpAddr>().ok(),
    };
    if ui.bind_addr.is_some_and(|management| {
        management.port() == request.config.port
            && host.is_some_and(|host| {
                host == management.ip() || host.is_unspecified() || management.ip().is_unspecified()
            })
    }) {
        return response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"error":"Gateway address conflicts with the management listener. Choose another port."}),
        );
    }
    // File validation, PBKDF2 and atomic persistence must not block an async worker.
    let state = ui.state.clone();
    let result = tokio::task::spawn_blocking(move || {
        state
            .access_settings()
            .save(request.revision, request.config, request.secret)
    })
    .await;
    match result {
        Ok(Ok(())) => read(State(ui)).await,
        Ok(Err(SaveError::Conflict)) => response(
            StatusCode::CONFLICT,
            json!({"error":"Configuration changed elsewhere. Reload before saving; restart the gateway if the configuration file was edited directly."}),
        ),
        Ok(Err(SaveError::Invalid(message))) => {
            response(StatusCode::UNPROCESSABLE_ENTITY, json!({"error":message}))
        }
        Ok(Err(SaveError::Io(_))) | Err(_) => response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"error":"Configuration could not be saved; existing settings were preserved"}),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn protects_all_management_routes_and_preserves_the_local_bootstrap_boundary() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            Directory(std::env::temp_dir().join(format!("transit-management-guard-{nonce}")));
        let state = transit_proxy::ProxyState::new();
        state
            .access_settings()
            .configure(
                directory.0.join("access.json"),
                AccessConfig {
                    host: "127.0.0.1".into(),
                    port: 8080,
                    auth_dir: directory.0.join("accounts").display().to_string(),
                    api_keys: vec![],
                    tls: Default::default(),
                    remote_management: Default::default(),
                },
            )
            .unwrap();
        let mut ui = UiServer::new(state.clone(), "127.0.0.1:8080".parse().unwrap(), true);
        ui.bind_addr = Some("0.0.0.0:15021".parse().unwrap());
        let app = routes()
            .route("/debug/config", get(|| async { "private" }))
            .route("/", get(|| async { "panel" }))
            .layer(axum::middleware::from_fn_with_state(ui.clone(), guard))
            .with_state(ui);
        let request = |path: &str, peer: &str, token: &str, origin: &str| {
            Request::builder()
                .uri(path)
                .header(header::HOST, "localhost:15021")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header("x-forwarded-for", "127.0.0.1")
                .header(header::ORIGIN, origin)
                .extension(ConnectInfo(peer.parse::<SocketAddr>().unwrap()))
                .body(Body::empty())
                .unwrap()
        };
        for (path, peer, key, origin, status) in [
            (
                "/admin/access",
                "127.0.0.1:1",
                "",
                "http://localhost:15021",
                StatusCode::UNAUTHORIZED,
            ),
            (
                "/debug/config",
                "127.0.0.1:1",
                "",
                "http://localhost:15021",
                StatusCode::UNAUTHORIZED,
            ),
            (
                "/admin/access",
                "192.0.2.1:1",
                "",
                "http://localhost:15021",
                StatusCode::FORBIDDEN,
            ),
        ] {
            assert_eq!(
                app.clone()
                    .oneshot(request(path, peer, key, origin))
                    .await
                    .unwrap()
                    .status(),
                status
            );
        }
        let mut local = request(
            "/admin/session",
            "127.0.0.1:1",
            "",
            "http://localhost:15021",
        );
        *local.method_mut() = axum::http::Method::POST;
        assert_eq!(
            app.clone().oneshot(local).await.unwrap().status(),
            StatusCode::OK
        );
        let token = state.access_settings().local_token().unwrap();
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/admin/access",
                    "127.0.0.1:1",
                    &token,
                    "http://localhost:15021"
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/admin/access",
                    "127.0.0.1:1",
                    &token,
                    "https://attacker.example"
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        let mut spoofed = request(
            "/admin/session",
            "127.0.0.1:1",
            "",
            "http://attacker.example:15021",
        );
        *spoofed.method_mut() = axum::http::Method::POST;
        spoofed
            .headers_mut()
            .insert(header::HOST, "attacker.example:15021".parse().unwrap());
        assert_eq!(
            app.clone().oneshot(spoofed).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
        let mut config = state.access_settings().view().unwrap().config;
        config.remote_management.allow_remote = true;
        let key = "test-management-key-for-remote-hosts";
        state
            .access_settings()
            .save(0, config, Some(key.into()))
            .unwrap();
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/admin/access",
                    "192.0.2.1:1",
                    key,
                    "http://localhost:15021"
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/admin/access",
                    "127.0.0.1:1",
                    &token,
                    "http://localhost:15021"
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        for _ in 0..5 {
            assert_eq!(
                app.clone()
                    .oneshot(request(
                        "/admin/access",
                        "192.0.2.2:1",
                        "wrong",
                        "http://localhost:15021"
                    ))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(
            app.clone()
                .oneshot(request(
                    "/admin/access",
                    "192.0.2.2:1",
                    key,
                    "http://localhost:15021"
                ))
                .await
                .unwrap()
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        let mut config = state.access_settings().view().unwrap().config;
        config.remote_management.disable_control_panel = true;
        state.access_settings().save(1, config, None).unwrap();
        assert_eq!(
            app.clone()
                .oneshot(request("/", "127.0.0.1:1", key, "http://localhost:15021"))
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            app.oneshot(request(
                "/admin/access",
                "127.0.0.1:1",
                key,
                "http://localhost:15021"
            ))
            .await
            .unwrap()
            .status(),
            StatusCode::OK
        );
    }
}
