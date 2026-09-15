//! Account-scoped management. Provider observations never come from backend configuration.
use super::{AccountSummary, LlmAccounts, OAuthAccount};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use hyper::{body::HttpBody, Body, Method};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path, time::Duration};

fn now() -> DateTime<Utc> {
    std::time::SystemTime::now().into()
}

pub(crate) struct AccountRequest {
    state: crate::ProxyState,
    id: String,
    result: Option<bool>,
}
impl Drop for AccountRequest {
    fn drop(&mut self) {
        self.state
            .llm_accounts()
            .record_account_result(&self.id, self.result.unwrap_or(false));
    }
}
impl AccountRequest {
    pub(crate) fn new(state: crate::ProxyState, id: String) -> Self {
        Self {
            state,
            id,
            result: None,
        }
    }
    pub(crate) fn body(self, body: Body, streaming: bool) -> Body {
        let stream = (body, self, HealthLines::default(), false);
        Body::wrap_stream(futures_util::stream::unfold(
            stream,
            move |(mut body, mut guard, mut lines, mut failed)| async move {
                match body.data().await {
                    Some(Ok(chunk)) => {
                        if streaming {
                            for line in lines.push(&chunk) {
                                if let Ok(v) = serde_json::from_str::<Value>(&line) {
                                    failed |= v.get("error").is_some_and(|v| !v.is_null())
                                        || matches!(
                                            v["type"].as_str(),
                                            Some("error" | "response.failed")
                                        );
                                }
                            }
                        }
                        Some((Ok::<_, hyper::Error>(chunk), (body, guard, lines, failed)))
                    }
                    Some(Err(error)) => Some((Err(error), (body, guard, lines, true))),
                    None => {
                        guard.result = Some(!failed);
                        drop(guard);
                        None
                    }
                }
            },
        ))
    }
}

#[derive(Default)]
struct HealthLines {
    bytes: Vec<u8>,
    skipping: bool,
}
impl HealthLines {
    fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        for byte in chunk {
            if *byte == b'\n' {
                if !self.skipping {
                    if let Some(data) = String::from_utf8_lossy(&self.bytes).strip_prefix("data:") {
                        lines.push(data.trim().to_string());
                    }
                }
                self.bytes.clear();
                self.skipping = false;
            } else if !self.skipping {
                if self.bytes.len() >= 1024 * 1024 {
                    self.bytes.clear();
                    self.skipping = true;
                } else {
                    self.bytes.push(*byte);
                }
            }
        }
        lines
    }
}

#[derive(Default)]
pub(super) struct Runtime {
    success: u64,
    failed: u64,
    buckets: BTreeMap<i64, (u64, u64)>,
    quota: Option<Value>,
    quota_error: Option<String>,
}

fn claims(account: &OAuthAccount) -> Value {
    account.document["id_token"]
        .as_str()
        .and_then(|s| s.split('.').nth(1))
        .and_then(|s| URL_SAFE_NO_PAD.decode(s).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null)
}

pub(super) fn metadata(account: &OAuthAccount) -> Value {
    let token = claims(account);
    let auth = &token["https://api.openai.com/auth"];
    json!({
        "disabled": account.disabled(),
        "plan_type": auth["chatgpt_plan_type"].as_str().or(account.document["plan_type"].as_str()),
        "renewal_at": instant(&auth["chatgpt_subscription_active_until"]),
        "file_name": format!("{}.json", account.id),
    })
}

impl OAuthAccount {
    pub fn disabled(&self) -> bool {
        self.document["disabled"].as_bool().unwrap_or(false)
    }
}

fn number(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str()?.parse().ok())
        .filter(|n| n.is_finite())
}
fn pick<'a>(v: &'a Value, snake: &str, camel: &str) -> &'a Value {
    if v[snake].is_null() {
        &v[camel]
    } else {
        &v[snake]
    }
}
fn instant(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        if let Ok(t) = DateTime::parse_from_rfc3339(s) {
            return Some(t.to_utc().to_rfc3339());
        }
    }
    number(v)
        .and_then(|n| {
            DateTime::from_timestamp(
                if n > 1e12 {
                    (n / 1000.) as i64
                } else {
                    n as i64
                },
                0,
            )
        })
        .map(|t| t.to_rfc3339())
}

impl LlmAccounts {
    pub async fn account_models(&self, id: &str) -> Result<Vec<Value>, String> {
        let mut account = self.get(id).ok_or("Account not found")?;
        let (method, path, body) = match account.provider() {
            "codex" => (
                Method::GET,
                "/backend-api/codex/models?client_version=999.0.0",
                None,
            ),
            "antigravity" => (
                Method::POST,
                "/v1internal:fetchAvailableModels",
                Some(json!({"project":account.document["project_id"]})),
            ),
            _ => (Method::GET, "/v1/models?limit=1000", None),
        };
        let mut result = provider_request(&account, method.clone(), path, body.clone()).await;
        if result
            .as_ref()
            .is_err_and(|e| e == "Provider returned HTTP 401")
            && account.summary().can_refresh
        {
            self.refresh(id).await?;
            account = self.get(id).ok_or("Account not found")?;
            result = provider_request(&account, method, path, body).await;
        }
        let payload = result?;
        if self.get(id).ok_or("Account not found")?.revision != account.revision {
            return Err("Account changed; reload the model catalog".into());
        }
        if account.provider() == "antigravity" {
            parse_antigravity_models(&payload)
        } else {
            parse_model_catalog(&payload)
        }
    }

    pub(super) fn account_summary(
        &self,
        account: &OAuthAccount,
        directory: Option<&Path>,
    ) -> AccountSummary {
        let mut summary = account.summary();
        let receipt = directory
            .and_then(|d| std::fs::read(d.join(format!("{}.reset", account.id))).ok())
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        if let Some(receipt) = receipt {
            summary.details["pending_reset_credit_id"] = receipt["requests"]
                .as_object()
                .and_then(|requests| requests.values().find(|r| r["status"] == "pending"))
                .map(|r| r["credit_id"].clone())
                .unwrap_or(Value::Null);
        }
        if let Some(file) =
            directory.and_then(|d| std::fs::metadata(d.join(format!("{}.json", account.id))).ok())
        {
            summary.details["file_size"] = json!(file.len());
            summary.details["modified_at"] = file
                .modified()
                .ok()
                .map(DateTime::<Utc>::from)
                .map(|t| json!(t.to_rfc3339()))
                .unwrap_or(Value::Null);
        }
        let runtime = self.runtime.lock().unwrap();
        let empty = Runtime::default();
        let r = runtime.get(&account.id).unwrap_or(&empty);
        let bucket = now().timestamp() / 300;
        let recent: Vec<_> = ((bucket - 23)..=bucket).map(|b| {
            let (success, failed) = r.buckets.get(&b).copied().unwrap_or_default();
            json!({"time": DateTime::from_timestamp(b * 300, 0).map(|t| t.to_rfc3339()), "success": success, "failed": failed})
        }).collect();
        summary.details["health"] = json!({"success":r.success,"failed":r.failed,"recent_requests":recent,"scope":"since_start"});
        summary.details["quota"] = json!(r.quota);
        summary.details["quota_error"] = json!(r.quota_error);
        summary
    }

    pub fn record_account_result(&self, id: &str, success: bool) {
        if self.get(id).is_none() {
            return;
        }
        let mut runtime = self.runtime.lock().unwrap();
        let r = runtime.entry(id.to_owned()).or_default();
        let bucket = now().timestamp() / 300;
        r.buckets.retain(|key, _| *key >= bucket - 23);
        let counts = r.buckets.entry(bucket).or_default();
        if success {
            r.success = r.success.saturating_add(1);
            counts.0 += 1;
        } else {
            r.failed = r.failed.saturating_add(1);
            counts.1 += 1;
        }
    }

    pub async fn set_disabled(
        &self,
        id: &str,
        revision: u64,
        disabled: bool,
    ) -> Result<AccountSummary, String> {
        let _guard = self.mutation.lock().await;
        let mut account = self.get(id).ok_or("Account not found")?;
        if account.revision != revision {
            return Err("Account changed; reload before saving".into());
        }
        account.document["disabled"] = json!(disabled);
        self.save_locked(account)
    }

    pub async fn delete(&self, id: &str, revision: u64) -> Result<(), String> {
        let _reset = self.reset_lock.lock().await;
        let _guard = self.mutation.lock().await;
        let mut data = self.data.write().unwrap();
        let account = data.accounts.get(id).ok_or("Account not found")?;
        if account.revision != revision {
            return Err("Account changed; reload before deleting".into());
        }
        let path = data
            .directory
            .as_ref()
            .ok_or("OAuth management is not configured")?
            .join(format!("{id}.json"));
        std::fs::remove_file(path).map_err(|_| "Cannot delete OAuth account file")?;
        data.accounts.remove(id);
        self.runtime.lock().unwrap().remove(id);
        if let Some(directory) = &data.directory {
            let receipt = directory.join(format!("{id}.reset"));
            if receipt.exists() {
                std::fs::remove_file(receipt)
                    .map_err(|_| "Account removed but reset receipt cleanup failed")?;
            }
        }
        Ok(())
    }

    pub async fn account_quota(&self, id: &str) -> Result<Value, String> {
        let mut account = self.get(id).ok_or("Account not found")?;
        let mut result = fetch_quota(&account).await;
        if result
            .as_ref()
            .is_err_and(|e| e == "Provider returned HTTP 401")
            && account.summary().can_refresh
        {
            if let Err(error) = self.refresh(id).await {
                if self.get(id).is_some() {
                    self.runtime
                        .lock()
                        .unwrap()
                        .entry(id.into())
                        .or_default()
                        .quota_error = Some(error.clone());
                }
                return Err(error);
            }
            account = self.get(id).ok_or("Account not found")?;
            result = fetch_quota(&account).await;
        }
        let _guard = self.mutation.lock().await;
        let current = self.get(id).ok_or("Account not found")?;
        if current.revision != account.revision {
            return Err("Account changed; refresh quota again".into());
        }
        let mut runtime = self.runtime.lock().unwrap();
        let r = runtime.entry(id.into()).or_default();
        match &result {
            Ok(quota) => {
                r.quota = Some(quota.clone());
                r.quota_error = None;
            }
            Err(error) => r.quota_error = Some(error.clone()),
        }
        result
    }

    pub async fn reset_account_quota(
        &self,
        id: &str,
        request_id: &str,
        credit_id: &str,
    ) -> Result<Value, String> {
        if request_id.len() != 36
            || !request_id.bytes().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            return Err("A UUID redeem_request_id is required".into());
        }
        if credit_id.trim().is_empty()
            || credit_id.len() > 256
            || credit_id.chars().any(char::is_control)
        {
            return Err("A valid credit_id is required".into());
        }
        let _guard = self.reset_lock.lock().await;
        let account = self.get(id).ok_or("Account not found")?;
        if account.provider() != "codex" {
            return Err("Quota reset is only supported for Codex".into());
        }
        let directory = self
            .data
            .read()
            .unwrap()
            .directory
            .clone()
            .ok_or("OAuth management is not configured")?;
        let receipt_path = directory.join(format!("{id}.reset"));
        let mut receipts: Value = if receipt_path.exists() {
            serde_json::from_slice(
                &std::fs::read(&receipt_path).map_err(|_| "Cannot read reset receipt")?,
            )
            .map_err(|_| "Invalid reset receipt; do not start another reset")?
        } else {
            json!({"requests":{}})
        };
        let request_id = prepare_reset_receipt(&mut receipts, request_id, credit_id)?;
        if receipts["requests"][&request_id]["status"] == "complete" {
            return Ok(receipts["requests"][&request_id]["result"].clone());
        }
        persist_receipt(&receipt_path, &receipts)?;
        // Persist before sending: a crash or lost browser session must reuse the upstream idempotency key.
        let path = "/backend-api/wham/rate-limit-reset-credits/consume";
        let body = json!({"redeem_request_id":request_id,"credit_id":credit_id});
        let mut response = provider_request(&account, Method::POST, path, Some(body.clone())).await;
        if response
            .as_ref()
            .is_err_and(|e| e == "Provider returned HTTP 401")
            && account.summary().can_refresh
        {
            self.refresh(id).await?;
            let refreshed = self.get(id).ok_or("Account not found")?;
            response = provider_request(&refreshed, Method::POST, path, Some(body)).await;
        }
        let result = response?;
        let code = reset_result_code(&result)?;
        let quota = self.account_quota(id).await;
        let result = json!({"reset_applied":code == "reset","code":code,"windows_reset":result["windows_reset"].as_u64().unwrap_or(0),"credit_id":credit_id,"redeem_request_id":request_id,"quota":quota.as_ref().ok(),"refresh_error":quota.err()});
        receipts["requests"][&request_id] =
            json!({"status":"complete","credit_id":credit_id,"result":result});
        persist_receipt(&receipt_path, &receipts)?;
        Ok(result)
    }
}

fn reset_result_code(result: &Value) -> Result<&str, String> {
    match result["code"].as_str() {
        Some(code @ ("reset" | "already_redeemed" | "nothing_to_reset" | "no_credit")) => Ok(code),
        _ => Err("Provider returned an unrecognized reset result; retry the same credit".into()),
    }
}

fn parse_model_catalog(payload: &Value) -> Result<Vec<Value>, String> {
    if payload["has_more"] == true {
        return Err("Provider model catalog is incomplete".into());
    }
    let items = payload
        .get("models")
        .or_else(|| payload.get("data"))
        .and_then(Value::as_array)
        .ok_or("Invalid provider model catalog")?;
    let mut seen = std::collections::BTreeSet::new();
    let models: Vec<_> = items
        .iter()
        .filter_map(|model| {
            if model
                .get("visibility")
                .and_then(Value::as_str)
                .is_some_and(|visibility| visibility != "list")
            {
                return None;
            }
            let id = model.get("slug").or_else(|| model.get("id"))?.as_str()?;
            if id.is_empty()
                || id.len() > 200
                || id.chars().any(char::is_control)
                || !seen.insert(id)
            {
                return None;
            }
            Some(json!({"id":id,"display_name":model["display_name"].as_str().unwrap_or(id)}))
        })
        .collect();
    if models.is_empty() {
        return Err("Provider returned no current models".into());
    }
    Ok(models)
}

fn parse_antigravity_models(payload: &Value) -> Result<Vec<Value>, String> {
    let models = payload["models"]
        .as_object()
        .ok_or("Invalid Antigravity model catalog")?;
    let current: Vec<&str> = payload["agentModelSorts"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|sort| sort["groups"].as_array().into_iter().flatten())
        .flat_map(|group| group["modelIds"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect();
    let aliases: std::collections::BTreeMap<&str, &str> = payload["deprecatedModelIds"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(old, reroute)| Some((reroute["newModelId"].as_str()?, old.as_str())))
        .collect();
    if current.is_empty() {
        return Err("Provider returned no current Antigravity model list".into());
    }
    let mut output = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for internal_id in current {
        let id = aliases.get(internal_id).copied().unwrap_or(internal_id);
        let model = models.get(id).or_else(|| models.get(internal_id));
        let Some(model) = model else { continue };
        if !model.is_object()
            || id.is_empty()
            || id.len() > 200
            || id.chars().any(char::is_control)
            || !seen.insert(id)
        {
            continue;
        }
        let display = model["displayName"]
            .as_str()
            .or_else(|| models.get(internal_id)?.get("displayName")?.as_str())
            .unwrap_or(id);
        output.push(json!({"id":id,"display_name":display}));
    }
    if output.is_empty() {
        return Err("Provider returned no current Antigravity models".into());
    }
    Ok(output)
}

fn prepare_reset_receipt(
    receipts: &mut Value,
    request_id: &str,
    credit_id: &str,
) -> Result<String, String> {
    if receipts.get("status").is_some() {
        // Preserve receipts written before individual credit selection was supported.
        let old_id = receipts["request_id"]
            .as_str()
            .ok_or("Invalid reset receipt")?
            .to_owned();
        *receipts = json!({"requests":{old_id:receipts.clone()}});
    }
    let requests = receipts["requests"]
        .as_object_mut()
        .ok_or("Invalid reset receipt")?;
    if let Some(previous) = requests.get(request_id) {
        if previous["credit_id"] != credit_id {
            return Err("Reset conflict: request ID belongs to another credit".into());
        }
        return Ok(request_id.into());
    }
    if let Some((pending_id, pending)) = requests.iter().find(|(_, r)| r["status"] == "pending") {
        if pending["credit_id"] != credit_id {
            return Err(
                "Reset conflict: another credit has an unresolved request; retry that credit first"
                    .into(),
            );
        }
        return Ok(pending_id.clone());
    }
    requests.insert(
        request_id.into(),
        json!({"status":"pending","credit_id":credit_id}),
    );
    Ok(request_id.into())
}

fn persist_receipt(path: &Path, value: &Value) -> Result<(), String> {
    use std::io::Write;
    let temporary = path.with_extension("reset.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> std::io::Result<()> {
        let mut file = options.open(&temporary)?;
        file.write_all(value.to_string().as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    result.map_err(|_| "Cannot persist reset receipt; retry only after storage is available".into())
}

async fn provider_request(
    account: &OAuthAccount,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let base = match account.provider() {
        "codex" => "https://chatgpt.com",
        "antigravity" => "https://daily-cloudcode-pa.googleapis.com",
        _ => "https://api.anthropic.com",
    };
    let url = format!("{base}{path}");
    // Deterministic local integration tests only; release builds have fixed provider origins.
    #[cfg(debug_assertions)]
    let url = match std::env::var("TRANSIT_TEST_OAUTH_API_BASE") {
        Ok(base) => {
            let uri: hyper::Uri = base.parse().map_err(|_| "Invalid OAuth test origin")?;
            if uri.scheme_str() != Some("http")
                || !matches!(uri.host(), Some("127.0.0.1" | "[::1]"))
                || uri.path() != "/"
            {
                return Err("OAuth test origin must be HTTP loopback without a path".into());
            }
            format!("{}{path}", base.trim_end_matches('/'))
        }
        Err(_) => url,
    };
    let mut request = hyper::Request::builder()
        .method(method)
        .uri(url)
        .header(
            "authorization",
            format!("Bearer {}", account.access_token()),
        )
        .header("content-type", "application/json")
        .header("accept", "application/json");
    if account.provider() == "codex" {
        let token = claims(account);
        let account_id = account.document["account_id"]
            .as_str()
            .or(token["https://api.openai.com/auth"]["chatgpt_account_id"].as_str());
        if let Some(id) = account_id {
            request = request.header("Chatgpt-Account-Id", id);
        }
        request = request
            .header("OpenAI-Beta", "codex-1")
            .header("Originator", "Transit")
            .header("user-agent", concat!("transit/", env!("CARGO_PKG_VERSION")));
    } else if account.provider() == "antigravity" {
        request = request.header("user-agent", "antigravity/hub/2.9.1 darwin/arm64");
    } else {
        request = request
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("anthropic-version", "2023-06-01");
    }
    let request = request
        .body(Body::from(body.map(|v| v.to_string()).unwrap_or_default()))
        .map_err(|_| "Cannot build provider request")?;
    let connector = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots();
    #[cfg(debug_assertions)]
    let connector = connector.https_or_http();
    #[cfg(not(debug_assertions))]
    let connector = connector.https_only();
    let client = hyper::Client::builder().build::<_, Body>(connector.enable_http1().build());
    tokio::time::timeout(Duration::from_secs(8), async {
        let mut response = client
            .request(request)
            .await
            .map_err(|_| "Provider connection failed".to_string())?;
        if !response.status().is_success() {
            return Err(format!(
                "Provider returned HTTP {}",
                response.status().as_u16()
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.body_mut().data().await {
            let chunk = chunk.map_err(|_| "Provider response failed")?;
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err("Provider response exceeds 1 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        if bytes.is_empty() {
            return Ok(json!({}));
        }
        serde_json::from_slice(&bytes).map_err(|_| "Invalid provider JSON response".into())
    })
    .await
    .map_err(|_| {
        "Provider request timed out; a reset may have completed, retry with the same request ID"
            .to_string()
    })?
}

fn codex_windows(payload: &Value) -> Vec<Value> {
    let mut groups = vec![
        (String::new(), pick(payload, "rate_limit", "rateLimit")),
        (
            "code-review".into(),
            pick(payload, "code_review_rate_limit", "codeReviewRateLimit"),
        ),
    ];
    if let Some(extra) = pick(payload, "additional_rate_limits", "additionalRateLimits").as_array()
    {
        for group in extra {
            groups.push((
                pick(group, "limit_name", "limitName")
                    .as_str()
                    .or(pick(group, "metered_feature", "meteredFeature").as_str())
                    .unwrap_or("additional")
                    .into(),
                pick(group, "rate_limit", "rateLimit"),
            ));
        }
    }
    let mut windows = Vec::new();
    for (group, limit) in groups {
        for (snake, camel, fallback) in [
            ("primary_window", "primaryWindow", 18000.),
            ("secondary_window", "secondaryWindow", 604800.),
        ] {
            let w = pick(limit, snake, camel);
            if !w.is_object() {
                continue;
            }
            let seconds =
                number(pick(w, "limit_window_seconds", "limitWindowSeconds")).unwrap_or(fallback);
            let used =
                number(pick(w, "used_percent", "usedPercent")).filter(|v| *v >= 0. && *v <= 100.);
            let reset = instant(pick(w, "reset_at", "resetAt")).or_else(|| {
                number(pick(w, "reset_after_seconds", "resetAfterSeconds"))
                    .filter(|n| *n >= 0.)
                    .and_then(|n| {
                        now().checked_add_signed(chrono::Duration::seconds(n.min(31536000.) as i64))
                    })
                    .map(|t| t.to_rfc3339())
            });
            windows.push(json!({"id":format!("{group}-{snake}"),"name":group,"window_seconds":seconds,"used_percent":used,"reset_at":reset}));
        }
    }
    windows
}

fn antigravity_remaining_fraction(value: &Value) -> Option<f64> {
    let mut value = pick(value, "remainingFraction", "remaining_fraction");
    if value.is_object() {
        value = pick(value, "remainingFraction", "remaining_fraction");
    }
    let mut remaining = number(value)?;
    if remaining > 1.0 && remaining <= 100.0 {
        remaining /= 100.0;
    }
    (0.0..=1.0).contains(&remaining).then_some(remaining)
}

fn antigravity_used_percent(remaining: f64) -> f64 {
    (((1.0 - remaining) * 100.0).clamp(0.0, 100.0) * 100.0).round() / 100.0
}

fn antigravity_subscription(payload: &Value) -> (Option<String>, Option<String>) {
    let paid = &payload["paidTier"];
    let current = &payload["currentTier"];
    let plan = [paid, current].into_iter().find_map(|tier| {
        tier["name"]
            .as_str()
            .or(tier["id"].as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    });
    let renewal = [paid, current, payload].into_iter().find_map(|value| {
        [
            "renewalTime",
            "renewal_time",
            "subscriptionRenewalTime",
            "subscription_renewal_time",
        ]
        .into_iter()
        .find_map(|key| instant(&value[key]))
    });
    (plan, renewal)
}

fn antigravity_summary_windows(payload: &Value) -> Vec<Value> {
    let groups = payload.get("groups").and_then(Value::as_array).or_else(|| {
        payload
            .get("response")
            .and_then(|value| value.get("groups"))
            .and_then(Value::as_array)
    });
    let Some(groups) = groups else {
        return Vec::new();
    };
    let mut windows = Vec::new();
    for (group_index, group) in groups.iter().enumerate() {
        let group_name = group
            .get("displayName")
            .or_else(|| group.get("display_name"))
            .or_else(|| group.get("name"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Antigravity");
        let group_description = group
            .get("description")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty());
        let Some(buckets) = group.get("buckets").and_then(Value::as_array) else {
            continue;
        };
        for (bucket_index, bucket) in buckets.iter().enumerate() {
            let remaining = antigravity_remaining_fraction(bucket).or_else(|| {
                bucket
                    .get("remaining")
                    .and_then(antigravity_remaining_fraction)
            });
            let Some(remaining) = remaining else { continue };
            let marker = format!(
                "{} {} {} {}",
                bucket["window"].as_str().unwrap_or(""),
                bucket["window_type"].as_str().unwrap_or(""),
                bucket["bucketId"]
                    .as_str()
                    .or(bucket["bucket_id"].as_str())
                    .or(bucket["id"].as_str())
                    .unwrap_or(""),
                bucket["displayName"]
                    .as_str()
                    .or(bucket["display_name"].as_str())
                    .or(bucket["name"].as_str())
                    .unwrap_or(""),
            )
            .to_ascii_lowercase();
            let window_seconds = if marker.contains("5h")
                || marker.contains("5-hour")
                || marker.contains("five hour")
                || marker.contains("hour")
            {
                18000.0
            } else if marker.contains("week")
                || marker.contains("7d")
                || marker.contains("seven day")
            {
                604800.0
            } else {
                number(pick(bucket, "window_seconds", "windowSeconds")).unwrap_or(0.0)
            };
            let bucket_id = bucket["bucketId"]
                .as_str()
                .or(bucket["bucket_id"].as_str())
                .or(bucket["id"].as_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("bucket");
            let bucket_label = bucket["displayName"]
                .as_str()
                .or(bucket["display_name"].as_str())
                .or(bucket["name"].as_str())
                .unwrap_or_else(|| {
                    if window_seconds == 604800.0 {
                        "Weekly Limit"
                    } else if window_seconds == 18000.0 {
                        "Five Hour Limit"
                    } else {
                        "Quota"
                    }
                });
            let mut window = json!({
                "id": format!("antigravity-summary-{group_index}-{bucket_index}-{bucket_id}"),
                "name": group_name,
                "window": "summary",
                "window_label": bucket_label,
                "window_seconds": window_seconds,
                "used_percent": antigravity_used_percent(remaining),
                "reset_at": instant(pick(bucket, "resetTime", "reset_time")),
            });
            if let Some(description) = group_description {
                window["group_description"] = json!(description);
            }
            windows.push(window);
        }
    }
    windows
}

fn reset_credits(payload: &Value) -> Result<Value, String> {
    if !payload.is_object()
        || ![
            "credits",
            "available_count",
            "availableCount",
            "applicable_available_count",
            "applicableAvailableCount",
        ]
        .iter()
        .any(|k| payload.get(k).is_some())
    {
        return Err("Invalid reset credit response".into());
    }
    let credits: Vec<_> = payload["credits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["status"] == "available")
        .map(|c| {
            let expires_at = instant(pick(c, "expires_at", "expiresAt"));
            json!({"id":c["id"].as_str(),"expires_at":expires_at,"reset_type":c["reset_type"].as_str(),"title":c["title"].as_str()})
        })
        .collect();
    let available = number(pick(payload, "available_count", "availableCount")).filter(|n| *n >= 0.);
    Ok(
        json!({"available_count":available.or_else(|| payload["credits"].is_array().then_some(credits.len() as f64)),"applicable_count":number(pick(payload,"applicable_available_count","applicableAvailableCount")).filter(|n| *n >= 0.),"credits":credits}),
    )
}

async fn fetch_quota(account: &OAuthAccount) -> Result<Value, String> {
    if account.provider() == "antigravity" {
        let project = account
            .document
            .get("project_id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or("Antigravity OAuth account has no project_id")?;
        let (payload, subscription) = tokio::join!(
            provider_request(
                account,
                Method::POST,
                "/v1internal:retrieveUserQuotaSummary",
                Some(json!({"project": project})),
            ),
            provider_request(
                account,
                Method::POST,
                "/v1internal:loadCodeAssist",
                Some(json!({"metadata":{"ideType":"ANTIGRAVITY"}})),
            )
        );
        let payload = payload?;
        if !payload.is_object() {
            return Err("Invalid Antigravity usage response".into());
        }
        let windows = antigravity_summary_windows(&payload);
        if windows.is_empty() {
            return Err("Provider returned no Antigravity quota groups".into());
        }
        let (plan_type, renewal_at, profile_error) = match subscription {
            Ok(profile) if profile.is_object() => {
                let (plan, renewal) = antigravity_subscription(&profile);
                (plan, renewal, None)
            }
            Ok(_) => (
                metadata(account)["plan_type"].as_str().map(str::to_string),
                None,
                Some("Invalid Antigravity subscription response".to_string()),
            ),
            Err(error) => (
                metadata(account)["plan_type"].as_str().map(str::to_string),
                None,
                Some(error),
            ),
        };
        let mut quota = json!({
            "observed_at": now().to_rfc3339(),
            "source": "provider_summary",
            "windows": windows,
            "plan_type": plan_type,
            "renewal_at": renewal_at,
        });
        if let Some(error) = profile_error {
            quota["profile_error"] = json!(error);
        }
        return Ok(quota);
    }
    let path = if account.provider() == "codex" {
        "/backend-api/wham/usage"
    } else {
        "/api/oauth/usage"
    };
    let payload = provider_request(account, Method::GET, path, None).await?;
    if !payload.is_object() {
        return Err("Invalid usage response".into());
    }
    let mut quota = json!({"observed_at":now().to_rfc3339(),"source":"provider","windows":[],"plan_type":metadata(account)["plan_type"],"renewal_at":metadata(account)["renewal_at"]});
    if account.provider() == "codex" {
        let windows = codex_windows(&payload);
        if windows.is_empty() {
            return Err("Provider returned no quota windows".into());
        }
        quota["windows"] = json!(windows);
        if let Some(plan) = pick(&payload, "plan_type", "planType").as_str() {
            quota["plan_type"] = json!(plan);
        }
        let fallback = reset_credits(pick(
            &payload,
            "rate_limit_reset_credits",
            "rateLimitResetCredits",
        ))
        .ok();
        let credits = provider_request(
            account,
            Method::GET,
            "/backend-api/wham/rate-limit-reset-credits",
            None,
        )
        .await
        .and_then(|v| reset_credits(&v));
        match credits {
            Ok(mut value) => {
                if let Some(fallback) = fallback {
                    if !fallback["applicable_count"].is_null() {
                        value["applicable_count"] = fallback["applicable_count"].clone();
                    }
                }
                quota["reset_credits"] = value;
            }
            Err(error) => {
                quota["reset_credits"] = json!(fallback);
                quota["credits_error"] = json!(error);
            }
        }
    } else {
        let windows: Vec<_> = payload.as_object().unwrap().iter().filter_map(|(key,v)| {
            if !v.is_object() || v.get("utilization").is_none() { return None; }
            Some(json!({"id":key,"name":key,"window_seconds":if key == "five_hour" {18000} else {604800},"used_percent":number(&v["utilization"]).filter(|n| *n >= 0. && *n <= 100.),"reset_at":instant(&v["resets_at"])}))
        }).collect();
        if windows.is_empty() {
            return Err("Provider returned no quota windows".into());
        }
        quota["windows"] = json!(windows);
        match provider_request(account, Method::GET, "/api/oauth/profile", None).await {
            Ok(profile) => {
                let plan = if profile["account"]["has_claude_max"] == true {
                    Some("Max")
                } else if profile["account"]["has_claude_pro"] == true {
                    Some("Pro")
                } else if profile["organization"]["organization_type"] == "claude_team" {
                    Some("Team")
                } else {
                    None
                };
                if let Some(plan) = plan {
                    quota["plan_type"] = json!(plan);
                }
            }
            Err(error) => quota["profile_error"] = json!(error),
        }
    }
    Ok(quota)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_credit_is_bound_to_durable_request_and_previous_results() {
        let mut receipts = json!({"requests":{}});
        assert_eq!(
            prepare_reset_receipt(&mut receipts, "request-1", "credit-2").unwrap(),
            "request-1"
        );
        assert_eq!(
            prepare_reset_receipt(&mut receipts, "new-browser-id", "credit-2").unwrap(),
            "request-1"
        );
        assert!(prepare_reset_receipt(&mut receipts, "request-1", "credit-1").is_err());
        assert!(prepare_reset_receipt(&mut receipts, "request-2", "credit-1").is_err());
        receipts["requests"]["request-1"] =
            json!({"status":"complete","credit_id":"credit-2","result":{"code":"reset"}});
        assert_eq!(
            prepare_reset_receipt(&mut receipts, "request-2", "credit-1").unwrap(),
            "request-2"
        );
        assert_eq!(
            prepare_reset_receipt(&mut receipts, "request-1", "credit-2").unwrap(),
            "request-1"
        );
        assert_eq!(receipts["requests"]["request-1"]["result"]["code"], "reset");
        let mut legacy = json!({"status":"pending","request_id":"old"});
        assert!(prepare_reset_receipt(&mut legacy, "new", "credit-1").is_err());
        assert_eq!(legacy["requests"]["old"]["status"], "pending");
    }

    #[test]
    fn reset_outcomes_require_a_known_provider_code() {
        for code in ["reset", "already_redeemed", "nothing_to_reset", "no_credit"] {
            assert_eq!(reset_result_code(&json!({"code":code})).unwrap(), code);
        }
        assert!(reset_result_code(&json!({"status":"ok"})).is_err());
        assert!(reset_result_code(&json!({"code":"future_code"})).is_err());
        let credits = reset_credits(
            &json!({"credits":[{"id":"credit-1","status":"available","expires_at":null}]}),
        )
        .unwrap();
        assert_eq!(credits["credits"][0]["id"], "credit-1");
        assert!(credits["credits"][0]["expires_at"].is_null());
    }

    #[test]
    fn model_catalog_only_exposes_names_and_supports_provider_shapes() {
        let catalog = parse_model_catalog(&json!({"models":[{"slug":"gpt-test","display_name":"GPT Test","visibility":"list","private_field":"hidden"},{"slug":"gpt-hidden","visibility":"hide"},{"slug":"gpt-test"}]})).unwrap();
        assert_eq!(
            catalog,
            vec![json!({"id":"gpt-test","display_name":"GPT Test"})]
        );
        assert_eq!(
            parse_model_catalog(&json!({"data":[{"id":"claude-test"}]})).unwrap()[0]["id"],
            "claude-test"
        );
        assert!(parse_antigravity_models(
            &json!({"models":{"gemini-test":{"displayName":"Gemini Test"}}})
        )
        .is_err());
        assert_eq!(
            parse_antigravity_models(&json!({
                "models":{
                    "gemini-current":{"displayName":"Gemini Current"},
                    "gemini-internal":{"displayName":"Gemini Current"},
                    "gemini-stale":{"displayName":"Gemini Stale"}
                },
                "agentModelSorts":[{"groups":[{"modelIds":["gemini-internal"]}]}],
                "deprecatedModelIds":{"gemini-current":{"newModelId":"gemini-internal"}}
            }))
            .unwrap(),
            vec![json!({"id":"gemini-current","display_name":"Gemini Current"})]
        );
        assert!(parse_model_catalog(&json!({"error":"bad"})).is_err());
        assert!(parse_antigravity_models(&json!({"models":[]})).is_err());
        assert!(parse_model_catalog(&json!({"data":[],"has_more":true})).is_err());
    }

    #[test]
    fn antigravity_summary_quota_preserves_group_and_window_labels() {
        let windows = antigravity_summary_windows(&json!({
            "response": {
                "groups": [{
                    "name": "Gemini Models",
                    "description": "Models within this group: Gemini Flash, Gemini Pro",
                    "buckets": [
                        {"id": "gemini-weekly", "name": "Weekly Limit Remaining", "window": "weekly", "remaining_fraction": 0.8},
                        {"id": "gemini-5h", "name": "Five Hour Limit Remaining", "window": "5h", "remaining_fraction": 0.5}
                    ]
                }]
            }
        }));
        assert_eq!(windows.len(), 2);
        assert!(windows
            .iter()
            .all(|window| window["name"] == "Gemini Models"));
        assert!(windows.iter().all(|window| {
            window["group_description"] == "Models within this group: Gemini Flash, Gemini Pro"
        }));
        assert_eq!(
            windows
                .iter()
                .find(|window| window["window_seconds"] == 604800.0)
                .unwrap()["window_label"],
            "Weekly Limit Remaining"
        );
        assert_eq!(
            windows
                .iter()
                .find(|window| window["window_seconds"] == 604800.0)
                .unwrap()["used_percent"],
            20.0
        );
        assert_eq!(
            windows
                .iter()
                .find(|window| window["window_seconds"] == 18000.0)
                .unwrap()["used_percent"],
            50.0
        );
        assert_eq!(
            antigravity_subscription(&json!({
                "currentTier":{"id":"free-tier","name":"Antigravity"},
                "paidTier":{"id":"g1-pro-tier","name":"Google AI Pro"}
            })),
            (Some("Google AI Pro".into()), None)
        );
    }
    struct Directory(std::path::PathBuf);
    impl Directory {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                    "transit-account-management-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn account() -> OAuthAccount {
        OAuthAccount {
            id: "account".into(),
            backend: "oauth".into(),
            document: json!({"type":"codex","access_token":"private-token","email":"user@example.test"}),
            models: vec![],
            revision: 0,
        }
    }

    #[tokio::test]
    async fn disable_delete_and_revision_guards_survive_restart() {
        let directory = Directory::new();
        let store = LlmAccounts::default();
        store
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        store.save(account()).await.unwrap();
        let backend: transit_core::Backend = serde_json::from_value(
            json!({"name":"oauth","type":"llm","provider":"openai","models":[]}),
        )
        .unwrap();
        assert!(store.select(&backend, None).is_some());
        assert!(store.set_disabled("account", 0, true).await.is_err());
        store.set_disabled("account", 1, true).await.unwrap();
        assert!(store.select(&backend, None).is_none());
        let summary = serde_json::to_value(store.list()).unwrap();
        assert_eq!(summary[0]["disabled"], true);
        assert!(summary[0]["file_size"].as_u64().unwrap() > 0);
        assert!(summary[0]["modified_at"].is_string());
        assert!(!summary.to_string().contains("private-token"));
        let restarted = LlmAccounts::default();
        restarted
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        assert!(restarted.get("account").unwrap().disabled());
        assert!(restarted.delete("account", 1).await.is_err());
        assert!(directory.0.join("account.json").exists());
        restarted.delete("account", 2).await.unwrap();
        assert!(!directory.0.join("account.json").exists());
        assert!(restarted.get("account").is_none());
    }

    #[test]
    fn quota_parsing_preserves_unknown_and_additional_windows() {
        let windows = codex_windows(
            &json!({"rate_limit":{"primary_window":{"used_percent":"0","limit_window_seconds":18000,"reset_at":1900000000},"secondary_window":{"used_percent":null,"limit_window_seconds":604800}},"additional_rate_limits":[{"limit_name":"gpt-reserve","rate_limit":{"secondary_window":{"used_percent":98,"reset_after_seconds":0}}}]}),
        );
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0]["used_percent"], 0.0);
        assert!(windows[1]["used_percent"].is_null());
        assert_eq!(windows[2]["name"], "gpt-reserve");
        assert!(windows[2]["reset_at"].is_string());
        assert!(
            codex_windows(&json!({"rate_limit":{"primary_window":{"used_percent":101}}}))[0]
                ["used_percent"]
                .is_null()
        );
        assert!(reset_credits(&json!({})).is_err());
        let credits = reset_credits(&json!({"available_count":0,"credits":[{"status":"consumed","expires_at":"2030-01-01T00:00:00Z"}]})).unwrap();
        assert_eq!(credits["available_count"], 0.0);
        assert_eq!(credits["credits"], json!([]));
        assert!(instant(&json!("invalid")).is_none());
        assert_eq!(
            instant(&json!("2030-01-01T08:00:00+08:00")).unwrap(),
            "2030-01-01T00:00:00+00:00"
        );
    }

    #[tokio::test]
    async fn health_tracks_stream_completion_error_and_cancellation_per_account() {
        let directory = Directory::new();
        let state = crate::ProxyState::new();
        state
            .llm_accounts()
            .configure(&directory.0, "test-management-token-long-enough".into())
            .unwrap();
        state.llm_accounts().save(account()).await.unwrap();
        let body = AccountRequest::new(state.clone(), "account".into())
            .body(Body::from("data: {\"choices\":[]}\n\n"), true);
        hyper::body::to_bytes(body).await.unwrap();
        let body = AccountRequest::new(state.clone(), "account".into()).body(
            Body::from("data: {\"error\":{\"message\":\"failed\"}}\n\n"),
            true,
        );
        hyper::body::to_bytes(body).await.unwrap();
        drop(AccountRequest::new(state.clone(), "account".into()).body(Body::empty(), false));
        let result = serde_json::to_value(state.llm_accounts().list()).unwrap();
        assert_eq!(result[0]["health"]["success"], 1);
        assert_eq!(result[0]["health"]["failed"], 2);
        assert_eq!(
            result[0]["health"]["recent_requests"]
                .as_array()
                .unwrap()
                .len(),
            24
        );
        assert!(!result.to_string().contains("private-token"));
    }
}
