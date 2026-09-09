//! Persistent OAuth documents and per-account model rules.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[path = "oauth_login.rs"]
mod login;
pub use login::OAuthLogin;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelRule {
    pub model: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub reasoning_effort: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OAuthAccount {
    pub id: String,
    #[serde(default)]
    pub backend: String,
    pub document: Value,
    #[serde(default)]
    pub models: Vec<ModelRule>,
    #[serde(default)]
    pub revision: u64,
}

#[derive(Debug, Serialize)]
pub struct AccountSummary {
    pub id: String,
    pub backend: String,
    pub provider: String,
    pub email: String,
    pub expires_at: Option<Value>,
    pub models: Vec<ModelRule>,
    pub revision: u64,
    pub can_refresh: bool,
}

impl OAuthAccount {
    pub fn provider(&self) -> &str {
        self.document
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    pub fn access_token(&self) -> &str {
        self.document
            .get("access_token")
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    pub fn summary(&self) -> AccountSummary {
        AccountSummary {
            id: self.id.clone(),
            backend: self.backend.clone(),
            provider: self.provider().to_string(),
            email: self
                .document
                .get("email")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            expires_at: self.document.get("expired").cloned(),
            models: self.models.clone(),
            revision: self.revision,
            can_refresh: self
                .document
                .get("refresh_token")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.is_empty()),
        }
    }

    pub fn resolve_model<'a>(&'a self, requested: &'a str) -> Option<(&'a str, &'a str)> {
        let rule = self
            .models
            .iter()
            .find(|r| r.model == requested || (!r.alias.is_empty() && r.alias == requested));
        match rule {
            Some(r) if r.disabled => None,
            Some(r) => Some((&r.model, &r.reasoning_effort)),
            None => Some((requested, "")),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 120
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(
                "Account ID must contain 1–120 letters, digits, hyphens or underscores".into(),
            );
        }
        if !self.backend.is_empty() && self.backend.trim().is_empty() {
            return Err("Backend name cannot contain only whitespace".into());
        }
        if !matches!(self.provider(), "codex" | "claude") {
            return Err("OAuth file type must be codex or claude".into());
        }
        if self.access_token().is_empty() || self.access_token().bytes().any(|b| b < 32 || b == 127)
        {
            return Err("OAuth file requires a valid access_token".into());
        }
        let mut names = BTreeSet::new();
        for rule in &self.models {
            if rule.model.trim().is_empty() || !names.insert(rule.model.as_str()) {
                return Err("Model names must be nonempty and unique".into());
            }
            if !matches!(
                rule.reasoning_effort.as_str(),
                "" | "none" | "minimal" | "low" | "medium" | "high" | "xhigh"
            ) {
                return Err("Unsupported reasoning effort".into());
            }
        }
        for rule in &self.models {
            if !rule.alias.is_empty()
                && rule.alias != rule.model
                && !names.insert(rule.alias.as_str())
            {
                return Err("Model aliases must not collide with models or other aliases".into());
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct AccountData {
    directory: Option<PathBuf>,
    admin_token: Option<String>,
    local_session: bool,
    accounts: BTreeMap<String, OAuthAccount>,
    cursors: BTreeMap<String, usize>,
}

#[derive(Default)]
pub struct LlmAccounts {
    data: RwLock<AccountData>,
    mutation: tokio::sync::Mutex<()>,
    logins: std::sync::Mutex<BTreeMap<String, login::PendingLogin>>,
    login_results: std::sync::Mutex<BTreeMap<String, (std::time::Instant, Value)>>,
    #[cfg(test)]
    refresh_endpoint: Option<String>,
}

impl LlmAccounts {
    pub fn configure(&self, directory: &Path, admin_token: String) -> io::Result<()> {
        if admin_token.len() < 24 {
            return Err(io::Error::other(
                "LLM admin token must contain at least 24 characters",
            ));
        }
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
        }
        let mut accounts = BTreeMap::new();
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let account: OAuthAccount = serde_json::from_slice(&std::fs::read(&path)?)
                .map_err(|_| io::Error::other("Invalid saved LLM account JSON"))?;
            account.validate().map_err(io::Error::other)?;
            if path.file_stem().and_then(|s| s.to_str()) != Some(&account.id) {
                return Err(io::Error::other(
                    "Saved account ID does not match its filename",
                ));
            }
            accounts.insert(account.id.clone(), account);
        }
        *self.data.write().unwrap() = AccountData {
            directory: Some(directory.to_path_buf()),
            admin_token: Some(admin_token),
            local_session: false,
            accounts,
            cursors: BTreeMap::new(),
        };
        Ok(())
    }

    pub fn configure_local(&self, directory: &Path) -> io::Result<()> {
        self.configure(directory, login::random().map_err(io::Error::other)?)?;
        self.data.write().unwrap().local_session = true;
        Ok(())
    }

    pub fn local_session_token(&self) -> Option<String> {
        let data = self.data.read().unwrap();
        data.local_session
            .then(|| data.admin_token.clone())
            .flatten()
    }

    pub fn enabled(&self) -> bool {
        self.data.read().unwrap().directory.is_some()
    }

    pub fn authorized(&self, candidate: &str) -> bool {
        let data = self.data.read().unwrap();
        let Some(expected) = &data.admin_token else {
            return false;
        };
        // Compare all bytes, including unequal-length input, without an early prefix exit.
        let mut diff = expected.len() ^ candidate.len();
        for (i, b) in expected.bytes().enumerate() {
            diff |= (b ^ candidate.as_bytes().get(i).copied().unwrap_or(0)) as usize;
        }
        diff == 0
    }

    pub fn list(&self) -> Vec<AccountSummary> {
        self.data
            .read()
            .unwrap()
            .accounts
            .values()
            .map(OAuthAccount::summary)
            .collect()
    }
    pub fn get(&self, id: &str) -> Option<OAuthAccount> {
        self.data.read().unwrap().accounts.get(id).cloned()
    }
    pub fn for_backend(&self, backend: &str) -> Vec<OAuthAccount> {
        self.data
            .read()
            .unwrap()
            .accounts
            .values()
            .filter(|a| a.backend == backend)
            .cloned()
            .collect()
    }

    pub fn select(
        &self,
        backend: &xgate_core::Backend,
        model: Option<&str>,
    ) -> Option<OAuthAccount> {
        let mut data = self.data.write().unwrap();
        let candidates: Vec<_> = data
            .accounts
            .values()
            .filter(|a| {
                a.backend == backend.name
                    && model.is_none_or(|m| {
                        a.resolve_model(m)
                            .is_some_and(|(resolved, _)| backend.supports_model(Some(resolved)))
                    })
            })
            .cloned()
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let cursor = data.cursors.entry(backend.name.clone()).or_default();
        let selected = candidates[*cursor % candidates.len()].clone();
        *cursor = cursor.wrapping_add(1);
        Some(selected)
    }

    pub async fn save(&self, account: OAuthAccount) -> Result<AccountSummary, String> {
        let _guard = self.mutation.lock().await;
        self.save_locked(account)
    }

    fn save_locked(&self, mut account: OAuthAccount) -> Result<AccountSummary, String> {
        account.validate()?;
        let mut data = self.data.write().unwrap();
        let directory = data
            .directory
            .as_ref()
            .ok_or("OAuth management is not configured")?;
        if data.accounts.values().any(|existing| {
            !account.backend.is_empty()
                && existing.id != account.id
                && existing.backend == account.backend
                && existing.provider() != account.provider()
        }) {
            return Err("A backend cannot mix OAuth providers".into());
        }
        let revision = data
            .accounts
            .get(&account.id)
            .map(|a| a.revision)
            .unwrap_or(0);
        if account.revision != revision {
            return Err("Account changed; reload before saving".into());
        }
        account.revision = revision.checked_add(1).ok_or("Account revision overflow")?;
        let bytes = serde_json::to_vec_pretty(&account).map_err(|_| "Cannot encode account")?;
        let target = directory.join(format!("{}.json", account.id));
        let temporary = directory.join(format!("{}.tmp", account.id));
        let write = || -> io::Result<()> {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            std::fs::rename(&temporary, &target)?;
            Ok(())
        };
        write().map_err(|_| {
            "Cannot persist OAuth account; check directory permissions and temporary files"
        })?;
        let summary = account.summary();
        data.accounts.insert(account.id.clone(), account);
        Ok(summary)
    }

    pub async fn refresh(&self, id: &str) -> Result<AccountSummary, String> {
        // Serialize refresh and edits so rotated refresh tokens cannot be lost.
        let _guard = self.mutation.lock().await;
        let mut account = self.get(id).ok_or("Account not found")?;
        let refresh_token = account
            .document
            .get("refresh_token")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or("OAuth file has no refresh_token")?;
        let (endpoint, client_id, scope) = match account.provider() {
            "codex" => ("https://auth.openai.com/oauth/token", "app_EMoamEEZ73f0CkXaXp7hrann", "openid profile email"),
            _ => ("https://platform.claude.com/v1/oauth/token", "9d1c250a-e61b-44d9-88ed-5944d1962f5e", "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload"),
        };
        #[cfg(test)]
        let endpoint = self.refresh_endpoint.as_deref().unwrap_or(endpoint);
        let encode = |s: &str| -> String {
            s.bytes()
                .map(|b| {
                    if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                        (b as char).to_string()
                    } else {
                        format!("%{b:02X}")
                    }
                })
                .collect()
        };
        let (body, content_type) = if account.provider() == "codex" {
            (
                format!(
                    "grant_type=refresh_token&client_id={}&refresh_token={}&scope={}",
                    encode(client_id),
                    encode(refresh_token),
                    encode(scope)
                ),
                "application/x-www-form-urlencoded",
            )
        } else {
            (serde_json::json!({"grant_type":"refresh_token","client_id":client_id,"refresh_token":refresh_token,"scope":scope}).to_string(), "application/json")
        };
        let connector = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots();
        #[cfg(test)]
        let connector = connector.https_or_http();
        #[cfg(not(test))]
        let connector = connector.https_only();
        let connector = connector.enable_http1().build();
        let client = hyper::Client::builder().build::<_, hyper::Body>(connector);
        let request = hyper::Request::post(endpoint)
            .header("content-type", content_type)
            .header("accept", "application/json")
            .body(hyper::Body::from(body))
            .map_err(|_| "Cannot build OAuth refresh request")?;
        let fetch = async {
            use hyper::body::HttpBody;
            let mut response = client
                .request(request)
                .await
                .map_err(|_| "OAuth refresh transport failed".to_string())?;
            if !response.status().is_success() {
                return Err(format!(
                    "OAuth refresh rejected with HTTP {}",
                    response.status().as_u16()
                ));
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response.body_mut().data().await {
                let chunk = chunk.map_err(|_| "OAuth refresh body failed".to_string())?;
                if bytes.len() + chunk.len() > 1024 * 1024 {
                    return Err("OAuth refresh response is too large".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            serde_json::from_slice::<Value>(&bytes)
                .map_err(|_| "Invalid OAuth refresh response".to_string())
        };
        let value = tokio::time::timeout(std::time::Duration::from_secs(30), fetch)
            .await
            .map_err(|_| "OAuth refresh timed out")??;
        merge_refresh(&mut account, value)?;
        self.save_locked(account)
    }
}

fn merge_refresh(account: &mut OAuthAccount, value: Value) -> Result<(), String> {
    if !value
        .get("access_token")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty())
    {
        return Err("OAuth refresh response has no access_token".into());
    }
    for key in ["access_token", "refresh_token", "id_token"] {
        if let Some(token) = value
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            account.document[key] = Value::String(token.to_string());
        }
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "Invalid system clock")?
        .as_secs();
    let rfc3339 = |t: u64| chrono::DateTime::from_timestamp(t as i64, 0).map(|d| d.to_rfc3339());
    account.document["last_refresh"] = serde_json::json!(rfc3339(now));
    if let Some(seconds) = value.get("expires_in").and_then(Value::as_u64) {
        account.document["expired"] = serde_json::json!(rfc3339(now.saturating_add(seconds)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("xgate-oauth-{}-{nonce}", std::process::id())))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn account() -> OAuthAccount {
        OAuthAccount {
            id: "test-account".into(),
            backend: "chatgpt".into(),
            document: serde_json::json!({"type":"codex","access_token":"secret-access","refresh_token":"secret-refresh","extra":{"preserved":true}}),
            models: vec![ModelRule {
                model: "gpt-test".into(),
                alias: "friendly".into(),
                reasoning_effort: "high".into(),
                disabled: false,
            }],
            revision: 0,
        }
    }

    #[tokio::test]
    async fn persistence_preserves_document_and_rules_but_summary_redacts_tokens() {
        let dir = Directory::new();
        let store = LlmAccounts::default();
        store
            .configure(&dir.0, "test-admin-token-long-enough".into())
            .unwrap();
        let summary = store.save(account()).await.unwrap();
        assert_eq!(summary.revision, 1);
        assert!(!serde_json::to_string(&summary).unwrap().contains("secret-"));
        let restarted = LlmAccounts::default();
        restarted
            .configure(&dir.0, "test-admin-token-long-enough".into())
            .unwrap();
        let saved = restarted.get("test-account").unwrap();
        assert_eq!(saved.document, account().document);
        assert_eq!(saved.resolve_model("friendly"), Some(("gpt-test", "high")));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(dir.0.join("test-account.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        assert!(restarted.authorized("test-admin-token-long-enough"));
        assert!(!restarted.authorized("test-admin-token-long-enough-extra"));
        assert!(!restarted.authorized("test-admin-token-long-enougH"));
    }

    #[tokio::test]
    async fn stale_edits_and_invalid_documents_do_not_replace_saved_credentials() {
        let dir = Directory::new();
        let store = LlmAccounts::default();
        store
            .configure(&dir.0, "test-admin-token-long-enough".into())
            .unwrap();
        store.save(account()).await.unwrap();
        assert!(store
            .save(account())
            .await
            .unwrap_err()
            .contains("Account changed"));
        let mut bad = account();
        bad.id = "../escape".into();
        assert!(store.save(bad).await.is_err());
        let mut bad = account();
        bad.document["access_token"] = serde_json::json!("bad\nheader");
        assert!(store.save(bad).await.is_err());
        assert_eq!(store.get("test-account").unwrap().revision, 1);
        assert_eq!(
            store.get("test-account").unwrap().document,
            account().document
        );
    }

    #[test]
    fn disabled_aliases_and_conflicting_model_names_are_rejected() {
        let mut account = account();
        account.models[0].disabled = true;
        assert_eq!(account.resolve_model("gpt-test"), None);
        assert_eq!(account.resolve_model("friendly"), None);
        account.models.push(ModelRule {
            model: "friendly".into(),
            ..ModelRule::default()
        });
        assert!(account.validate().is_err());
    }

    #[test]
    fn token_rotation_preserves_refresh_token_when_provider_omits_it() {
        let mut account = account();
        merge_refresh(
            &mut account,
            serde_json::json!({"access_token":"rotated","expires_in":3600}),
        )
        .unwrap();
        assert_eq!(account.document["refresh_token"], "secret-refresh");
        assert_eq!(account.document["access_token"], "rotated");
        assert!(account.document["expired"].as_str().unwrap().contains('T'));
        assert_eq!(account.document["extra"]["preserved"], true);
    }

    #[tokio::test]
    async fn refresh_posts_provider_wire_format_and_persists_rotated_credentials() {
        use std::sync::{Arc, Mutex};
        let calls = Arc::new(Mutex::new(Vec::new()));
        let capture = calls.clone();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let service = hyper::service::make_service_fn(move |_| {
            let capture = capture.clone();
            async move {
                Ok::<_, std::convert::Infallible>(hyper::service::service_fn(
                    move |request: hyper::Request<hyper::Body>| {
                        let capture = capture.clone();
                        async move {
                            let content_type = request.headers()["content-type"]
                                .to_str()
                                .unwrap()
                                .to_string();
                            let bytes = hyper::body::to_bytes(request.into_body()).await.unwrap();
                            capture
                                .lock()
                                .unwrap()
                                .push((content_type, String::from_utf8(bytes.to_vec()).unwrap()));
                            Ok::<_, std::convert::Infallible>(hyper::Response::new(
                                hyper::Body::from(
                                    r#"{"access_token":"rotated-access","refresh_token":"rotated-refresh","expires_in":3600}"#,
                                ),
                            ))
                        }
                    },
                ))
            }
        });
        let server = tokio::spawn(hyper::Server::from_tcp(listener).unwrap().serve(service));
        for provider in ["codex", "claude"] {
            let directory = Directory::new();
            let store = LlmAccounts {
                refresh_endpoint: Some(format!("http://{addr}/token")),
                ..LlmAccounts::default()
            };
            store
                .configure(&directory.0, "test-admin-token-long-enough".into())
                .unwrap();
            let mut account = account();
            account.document["type"] = serde_json::json!(provider);
            account.document["refresh_token"] = serde_json::json!("refresh+A&B");
            store.save(account).await.unwrap();
            let summary = store.refresh("test-account").await.unwrap();
            assert_eq!(summary.revision, 2);
            let saved = store.get("test-account").unwrap();
            assert_eq!(saved.access_token(), "rotated-access");
            assert_eq!(saved.document["refresh_token"], "rotated-refresh");
            let disk: OAuthAccount = serde_json::from_slice(
                &std::fs::read(directory.0.join("test-account.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(disk.document, saved.document);
        }
        server.abort();
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].0, "application/x-www-form-urlencoded");
        assert!(calls[0].1.contains("refresh_token=refresh%2BA%26B"));
        assert!(calls[0]
            .1
            .contains("client_id=app_EMoamEEZ73f0CkXaXp7hrann"));
        assert_eq!(calls[1].0, "application/json");
        let claude: Value = serde_json::from_str(&calls[1].1).unwrap();
        assert_eq!(claude["grant_type"], "refresh_token");
        assert_eq!(claude["refresh_token"], "refresh+A&B");
        assert_eq!(claude["client_id"], "9d1c250a-e61b-44d9-88ed-5944d1962f5e");
    }
}
