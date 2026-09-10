use hyper::{body::HttpBody, Body, Client, Request, Uri};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use transit_proxy::{
    access_settings::{github_release_url, write_private},
    ProxyState,
};

const LIMIT: usize = 10 * 1024 * 1024;
const COMPATIBILITY: &str = "name=\"transit-management-api\" content=\"1\"";

#[derive(Default)]
struct Cached {
    repository: String,
    html: Option<String>,
    checked: Option<Instant>,
    error: Option<String>,
}

#[derive(Default)]
pub struct PanelAssets(Mutex<Cached>);

impl PanelAssets {
    pub fn start(self: Arc<Self>, state: ProxyState) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3 * 60 * 60));
            loop {
                interval.tick().await;
                let Some(remote) = state.access_settings().remote() else {
                    continue;
                };
                if !remote.disable_control_panel && !remote.disable_auto_update_panel {
                    self.update(&state, true).await;
                }
            }
        })
    }

    pub async fn status(&self, state: &ProxyState) -> Value {
        let cached = self.0.lock().await;
        if !state.access_settings().remote().is_some_and(|remote| {
            !remote.panel_github_repository.is_empty()
                && remote.panel_github_repository == cached.repository
        }) {
            return json!({"source":"bundled","error":null});
        }
        json!({"source":if cached.html.is_some(){"repository"}else{"bundled"},"error":cached.error})
    }

    pub async fn html(&self, state: &ProxyState) -> Option<String> {
        self.update(state, false).await;
        let remote = state.access_settings().remote()?;
        if remote.disable_control_panel {
            return None;
        }
        let cached = self.0.lock().await;
        (cached.repository == remote.panel_github_repository
            && !remote.panel_github_repository.is_empty())
        .then(|| cached.html.clone())
        .flatten()
    }

    async fn update(&self, state: &ProxyState, background: bool) {
        let Some(view) = state.access_settings().view() else {
            return;
        };
        let remote = &view.config.remote_management;
        if remote.disable_control_panel || remote.panel_github_repository.is_empty() {
            return;
        }
        let mut cached = self.0.lock().await;
        if cached.repository != remote.panel_github_repository {
            *cached = Cached {
                repository: remote.panel_github_repository.clone(),
                ..Default::default()
            };
        }
        let repository_hash =
            ring::digest::digest(&ring::digest::SHA256, cached.repository.as_bytes());
        let hash: String = repository_hash
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let path = std::path::Path::new(&view.config_file)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("panels")
            .join(format!("{hash}.html"));
        if cached.html.is_none() {
            if let Ok(html) = tokio::fs::read_to_string(&path).await {
                if html.len() <= LIMIT && html.contains(COMPATIBILITY) {
                    cached.html = Some(html);
                }
            }
        }
        if (!background && cached.html.is_some())
            || (background && remote.disable_auto_update_panel)
            || cached
                .checked
                .is_some_and(|last| last.elapsed() < Duration::from_secs(30))
        {
            return;
        }
        cached.checked = Some(Instant::now());
        let repository = cached.repository.clone();
        drop(cached);
        let result =
            tokio::time::timeout(Duration::from_secs(20), download_panel(&repository)).await;
        let mut cached = self.0.lock().await;
        if cached.repository != repository {
            return;
        }
        match result {
            Ok(Ok(html)) => {
                let bytes = html.as_bytes().to_vec();
                match tokio::task::spawn_blocking(move || write_private(&path, &bytes)).await {
                    Ok(Ok(())) => {
                        cached.html = Some(html);
                        cached.error = None;
                    }
                    _ => {
                        cached.error = Some(
                            "Could not cache the control panel; the existing panel is retained"
                                .into(),
                        )
                    }
                }
            }
            Ok(Err(error)) => cached.error = Some(error),
            Err(_) => {
                cached.error =
                    Some("Panel download timed out; the existing panel is retained".into())
            }
        }
    }
}

async fn download_panel(repository: &str) -> Result<String, String> {
    download_panel_with(repository, |url: String| async move { fetch(&url).await }).await
}

async fn download_panel_with<F, Fut>(repository: &str, get: F) -> Result<String, String>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<u8>, String>>,
{
    let release: Value = serde_json::from_slice(&get(github_release_url(repository)?).await?)
        .map_err(|_| "Invalid GitHub release response")?;
    let asset = release["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .find(|asset| asset["name"] == "management.html")
        })
        .ok_or("Release has no management.html asset")?;
    let bytes = get(asset["browser_download_url"]
        .as_str()
        .ok_or("Release asset has no download URL")?
        .to_string())
    .await?;
    if let Some(expected) = asset["digest"]
        .as_str()
        .and_then(|digest| digest.strip_prefix("sha256:"))
    {
        let digest = ring::digest::digest(&ring::digest::SHA256, &bytes);
        let actual: String = digest
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        if actual != expected {
            return Err("Panel asset checksum mismatch".into());
        }
    }
    let html = String::from_utf8(bytes).map_err(|_| "Panel must be UTF-8 HTML")?;
    if !html.contains(COMPATIBILITY) {
        return Err("Panel is not compatible with this project's management API".into());
    }
    Ok(html)
}

async fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();
    let client: Client<_, Body> = Client::builder().build(connector);
    let mut uri: Uri = url.parse().map_err(|_| "Invalid panel download URL")?;
    for _ in 0..5 {
        if uri.scheme_str() != Some("https")
            || !matches!(
                uri.host(),
                Some(
                    "api.github.com"
                        | "github.com"
                        | "objects.githubusercontent.com"
                        | "release-assets.githubusercontent.com"
                )
            )
            || uri.port_u16().is_some_and(|port| port != 443)
        {
            return Err("Panel download must use GitHub HTTPS".into());
        }
        let request = Request::get(uri.clone())
            .header("User-Agent", "transit-management-updater")
            .body(Body::empty())
            .map_err(|_| "Invalid panel request")?;
        let response = client
            .request(request)
            .await
            .map_err(|_| "Could not reach the panel repository")?;
        if response.status().is_redirection() {
            uri = response
                .headers()
                .get("location")
                .and_then(|location| location.to_str().ok())
                .and_then(|url| url.parse().ok())
                .ok_or("Invalid panel redirect")?;
            continue;
        }
        if !response.status().is_success() {
            return Err(format!(
                "Panel repository returned HTTP {}",
                response.status().as_u16()
            ));
        }
        let mut body = response.into_body();
        let mut bytes = Vec::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.map_err(|_| "Panel download interrupted")?;
            if bytes.len() + chunk.len() > LIMIT {
                return Err("Panel download exceeds 10 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        return Ok(bytes);
    }
    Err("Too many panel download redirects".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn release_download_selects_the_panel_and_checks_digest_and_compatibility() {
        let html = format!("<!doctype html><meta {COMPATIBILITY}><title>Project panel</title>");
        let digest = ring::digest::digest(&ring::digest::SHA256, html.as_bytes());
        let hash: String = digest
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        for (digest, contents, expected) in [
            (hash.clone(), html.clone(), true),
            ("0".repeat(64), html, false),
            (hash, "<html>different API</html>".into(), false),
        ] {
            let release = json!({"assets":[{"name":"other.html","browser_download_url":"https://github.com/wrong"},{"name":"management.html","browser_download_url":"https://github.com/owner/panel/releases/download/v1/management.html","digest":format!("sha256:{digest}")}]}).to_string().into_bytes();
            let result = download_panel_with("https://github.com/owner/panel", |url| {
                std::future::ready(match url.as_str() {
                    "https://api.github.com/repos/owner/panel/releases/latest" => {
                        Ok(release.clone())
                    }
                    "https://github.com/owner/panel/releases/download/v1/management.html" => {
                        Ok(contents.as_bytes().to_vec())
                    }
                    _ => panic!("Unexpected release URL"),
                })
            })
            .await;
            assert_eq!(result.is_ok(), expected);
        }
        let incompatible = json!({"assets":[{"name":"management.html","browser_download_url":"https://github.com/owner/panel/management.html"}]}).to_string().into_bytes();
        assert!(
            download_panel_with("https://github.com/owner/panel", |url| std::future::ready(
                Ok(if url.ends_with("/releases/latest") {
                    incompatible.clone()
                } else {
                    b"<html>different API</html>".to_vec()
                })
            ))
            .await
            .unwrap_err()
            .contains("not compatible")
        );
    }
    #[tokio::test]
    async fn cached_panel_survives_disabled_updates_and_bundled_fallback_is_truthful() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("transit-panel-{nonce}"));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let state = ProxyState::new();
        let repository = "https://github.com/owner/compatible-panel";
        let config = transit_proxy::access_settings::AccessConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            auth_dir: directory.join("accounts").display().to_string(),
            api_keys: vec![],
            tls: Default::default(),
            remote_management: transit_proxy::access_settings::RemoteManagement {
                panel_github_repository: repository.into(),
                disable_auto_update_panel: true,
                ..Default::default()
            },
        };
        state
            .access_settings()
            .configure(directory.join("access.json"), config)
            .unwrap();
        let digest = ring::digest::digest(&ring::digest::SHA256, repository.as_bytes());
        let hash: String = digest
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let html = format!("<!doctype html><meta {COMPATIBILITY}><title>Compatible panel</title>");
        write_private(
            &directory.join("panels").join(format!("{hash}.html")),
            html.as_bytes(),
        )
        .unwrap();
        let panel = PanelAssets::default();
        assert_eq!(panel.html(&state).await.as_deref(), Some(html.as_str()));
        panel.update(&state, true).await;
        assert!(panel.0.lock().await.checked.is_none());
        assert_eq!(panel.status(&state).await["source"], "repository");
        let mut config = state.access_settings().view().unwrap().config;
        config.remote_management.panel_github_repository.clear();
        state.access_settings().save(0, config, None).unwrap();
        assert!(panel.html(&state).await.is_none());
        assert_eq!(
            panel.status(&state).await,
            json!({"source":"bundled","error":null})
        );
        for url in [
            "http://github.com/a/b",
            "https://localhost/panel",
            "https://github.com:8443/a/b",
            "https://example.com/panel",
        ] {
            assert!(fetch(url).await.is_err());
        }
    }
}
