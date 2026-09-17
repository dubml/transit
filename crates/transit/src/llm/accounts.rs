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
#[path = "account_management.rs"]
mod management;
#[allow(unused_imports)]
pub(crate) use management::AccountRequest;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelRule {
    pub model: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub alias: String,
    #[serde(default = "keep_original_default")]
    pub keep_original: bool,
    #[serde(default)]
    pub reasoning_effort: String,
}

fn keep_original_default() -> bool {
    true
}

fn model_pattern_matches(pattern: &str, model: &str) -> bool {
    if let Some(expression) = model_regex(pattern) {
        return regex::RegexBuilder::new(expression)
            .case_insensitive(true)
            .build()
            .is_ok_and(|regex| regex.is_match(model));
    }
    let pattern = pattern.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    let (p, m) = (pattern.as_bytes(), model.as_bytes());
    let (mut i, mut j, mut star, mut retry) = (0, 0, None, 0);
    while j < m.len() {
        if i < p.len() && p[i] != b'*' && p[i] == m[j] {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == b'*' {
            star = Some(i);
            i += 1;
            retry = j;
        } else if let Some(index) = star {
            retry += 1;
            j = retry;
            i = index + 1;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == b'*' {
        i += 1;
    }
    i == p.len()
}

fn model_regex(pattern: &str) -> Option<&str> {
    pattern.strip_prefix("re:").or_else(|| {
        pattern
            .strip_prefix('/')
            .and_then(|value| value.strip_suffix('/'))
    })
}

fn is_model_pattern(model: &str) -> bool {
    model.contains('*') || model_regex(model).is_some()
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
    #[serde(flatten)]
    pub details: Value,
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
            details: management::metadata(self),
        }
    }

    pub fn resolve_model<'a>(&'a self, requested: &'a str) -> Option<(&'a str, &'a str)> {
        let rule = self
            .models
            .iter()
            .find(|r| r.model == requested || (!r.alias.is_empty() && r.alias == requested));
        let resolved = rule.map(|r| r.model.as_str()).unwrap_or(requested);
        if self.models.iter().any(|r| {
            r.disabled
                && (model_pattern_matches(&r.model, requested)
                    || model_pattern_matches(&r.model, resolved))
        }) {
            return None;
        }
        if rule.is_some_and(|r| {
            !r.alias.is_empty() && !r.keep_original && r.model == requested && r.alias != requested
        }) {
            return None;
        }
        Some((
            resolved,
            rule.map(|r| r.reasoning_effort.as_str()).unwrap_or(""),
        ))
    }

    pub fn public_model_names<'a>(&'a self, original: &'a str) -> Vec<&'a str> {
        let mut names = Vec::new();
        if self.resolve_model(original).is_some() {
            names.push(original);
        }
        for rule in &self.models {
            if rule.model == original
                && !rule.alias.is_empty()
                && rule.alias != original
                && self.resolve_model(&rule.alias).is_some()
            {
                names.push(&rule.alias);
            }
        }
        names
    }

    fn validate(&self) -> Result<(), String> {
        if self
            .document
            .get("disabled")
            .is_some_and(|v| !v.is_boolean())
        {
            return Err("OAuth disabled must be a boolean".into());
        }
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
        if !matches!(self.provider(), "codex" | "claude" | "antigravity") {
            return Err("OAuth file type must be codex, claude or antigravity".into());
        }
        if self.access_token().is_empty() || self.access_token().bytes().any(|b| b < 32 || b == 127)
        {
            return Err("OAuth file requires a valid access_token".into());
        }
        if self.provider() == "antigravity"
            && self
                .document
                .get("project_id")
                .and_then(Value::as_str)
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err("Antigravity OAuth file requires a project_id".into());
        }
        let mut names = BTreeSet::new();
        for rule in &self.models {
            if rule.model.trim().is_empty()
                || rule.model.trim() != rule.model
                || rule.model.chars().any(char::is_control)
                || !names.insert(rule.model.to_ascii_lowercase())
            {
                return Err("Model names must be nonempty and unique".into());
            }
            if is_model_pattern(&rule.model)
                && (!rule.disabled || !rule.alias.is_empty() || !rule.reasoning_effort.is_empty())
            {
                return Err("Patterns are only supported for model exclusion rules".into());
            }
            if let Some(expression) = model_regex(&rule.model) {
                regex::RegexBuilder::new(expression)
                    .case_insensitive(true)
                    .build()
                    .map_err(|_| "Invalid model exclusion regex")?;
            }
            if !matches!(
                rule.reasoning_effort.as_str(),
                "" | "none" | "minimal" | "low" | "medium" | "high" | "xhigh"
            ) {
                return Err("Unsupported reasoning effort".into());
            }
        }
        for rule in &self.models {
            if rule.alias.trim() != rule.alias
                || rule.alias.contains('*')
                || rule.alias.chars().any(char::is_control)
            {
                return Err(
                    "Model aliases must be exact, nonblank names without control characters".into(),
                );
            }
            if !rule.alias.is_empty()
                && !rule.alias.eq_ignore_ascii_case(&rule.model)
                && !names.insert(rule.alias.to_ascii_lowercase())
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
    runtime: std::sync::Mutex<BTreeMap<String, management::Runtime>>,
    reset_lock: tokio::sync::Mutex<()>,
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
        let data = self.data.read().unwrap();
        data.accounts
            .values()
            .map(|account| self.account_summary(account, data.directory.as_deref()))
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
        backend_name: &str,
        model: Option<&str>,
    ) -> Option<OAuthAccount> {
        let mut data = self.data.write().unwrap();
        let candidates: Vec<_> = data
            .accounts
            .values()
            .filter(|a| {
                a.backend == backend_name
                    && !a.disabled()
                    && model.is_none_or(|m| a.resolve_model(m).is_some())
            })
            .cloned()
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let cursor = data.cursors.entry(backend_name.to_string()).or_default();
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
        let identity_changed = data.accounts.get(&account.id).is_some_and(|existing| {
            existing.provider() != account.provider()
                || existing.document["account_id"] != account.document["account_id"]
                || existing.document["email"] != account.document["email"]
        });
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
        if identity_changed {
            self.runtime.lock().unwrap().remove(&account.id);
        }
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

        let (endpoint, client_id, client_secret, scope) = match account.provider() {
            "codex" => (
                "https://auth.openai.com/oauth/token",
                "app_EMoamEEZ73f0CkXaXp7hrann".to_string(),
                None,
                "openid profile email",
            ),
            "antigravity" => (
                "https://oauth2.googleapis.com/token",
                std::env::var("GOOGLE_OAUTH_CLIENT_ID")
                    .map_err(|_| "GOOGLE_OAUTH_CLIENT_ID is not set")?,
                Some(
                    std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
                        .map_err(|_| "GOOGLE_OAUTH_CLIENT_SECRET is not set")?,
                ),
                "",
            ),
            _ => (
                "https://platform.claude.com/v1/oauth/token",
                "9d1c250a-e61b-44d9-88ed-5944d1962f5e".to_string(),
                None,
                "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload",
            ),
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

        let (body, content_type) = if matches!(account.provider(), "codex" | "antigravity") {
            let mut pairs = vec![
                ("grant_type", "refresh_token"),
                ("client_id", client_id.as_str()),
                ("refresh_token", refresh_token),
            ];

            if !scope.is_empty() {
                pairs.push(("scope", scope));
            }

            if let Some(secret) = client_secret.as_deref() {
                pairs.push(("client_secret", secret));
            }

            (
                pairs
                    .into_iter()
                    .map(|(key, value)| format!("{}={}", encode(key), encode(value)))
                    .collect::<Vec<_>>()
                    .join("&"),
                "application/x-www-form-urlencoded",
            )
        } else {
            (
                serde_json::json!({
                "grant_type": "refresh_token",
                "client_id": client_id,
                "refresh_token": refresh_token,
                "scope": scope
            })
                    .to_string(),
                "application/json",
            )
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
    }}

fn merge_refresh(account: &mut OAuthAccount, value: Value) -> Result<(), String> {
    if value
        .get("access_token")
        .and_then(Value::as_str)
        .is_none_or(|s| s.is_empty())
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
