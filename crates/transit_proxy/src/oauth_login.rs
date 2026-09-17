//! Browser authorization with PKCE. Credentials and verifiers never leave the server.
use super::{merge_refresh, AccountSummary, LlmAccounts, OAuthAccount};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::{
    digest,
    rand::{SecureRandom, SystemRandom},
};
use serde::Serialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::time::{Duration, Instant};

const LIFETIME: Duration = Duration::from_secs(600);

struct Provider {
    authorize: &'static str,
    token: &'static str,
    client: Cow<'static, str>,
    client_secret: Option<Cow<'static, str>>,
    redirect: &'static str,
    scope: &'static str,
    pkce: bool,
}

fn provider(name: &str) -> Result<Provider, String> {
    match name {
        "codex" => Ok(Provider {
            authorize: "https://auth.openai.com/oauth/authorize",
            token: "https://auth.openai.com/oauth/token",
            client: Cow::Borrowed("app_EMoamEEZ73f0CkXaXp7hrann"),
            client_secret: None,
            redirect: "http://localhost:1455/auth/callback",
            scope: "openid email profile offline_access",
            pkce: true,
        }),

        "claude" => Ok(Provider {
            authorize: "https://claude.ai/oauth/authorize",
            token: "https://platform.claude.com/v1/oauth/token",
            client: Cow::Borrowed("9d1c250a-e61b-44d9-88ed-5944d1962f5e"),
            client_secret: None,
            redirect: "http://localhost:54545/callback",
            scope: "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload",
            pkce: true,
        }),

        "antigravity" => Ok(Provider {
            authorize: "https://accounts.google.com/o/oauth2/v2/auth",
            token: "https://oauth2.googleapis.com/token",
            client: Cow::Owned(
                std::env::var("GOOGLE_OAUTH_CLIENT_ID")
                    .map_err(|_| "GOOGLE_OAUTH_CLIENT_ID is not set")?,
            ),
            client_secret: Some(Cow::Owned(
                std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
                    .map_err(|_| "GOOGLE_OAUTH_CLIENT_SECRET is not set")?,
            )),
            redirect: "http://localhost:51121/oauth-callback",
            scope: "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/cclog https://www.googleapis.com/auth/experimentsandconfigs",
            pkce: false,
        }),

        _ => Err("Unsupported OAuth provider".into()),
    }
}

pub(super) struct PendingLogin {
    provider: String,
    backend: String,
    verifier: String,
    state: String,
    redirect: String,
    created: Instant,
}

#[derive(Debug, Serialize)]
pub struct OAuthLogin {
    pub id: String,
    pub authorization_url: String,
    pub redirect_uri: String,
    pub expires_in: u64,
}

pub(super) fn random() -> Result<String, String> {
    let mut bytes = [0; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| "Cannot generate OAuth randomness")?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

impl LlmAccounts {
    pub fn start_login(&self, name: &str, backend: &str) -> Result<OAuthLogin, String> {
        self.start_login_with_redirect(name, backend, None)
    }

    pub fn start_login_with_redirect(
        &self,
        name: &str,
        backend: &str,
        redirect_override: Option<&str>,
    ) -> Result<OAuthLogin, String> {
        if !self.enabled() {
            return Err("OAuth management is not configured".into());
        }
        let config = provider(name)?;
        let redirect = redirect_override.unwrap_or(config.redirect).to_string();
        let id = random()?;
        let state = random()?;
        let verifier = if config.pkce {
            random()?
        } else {
            String::new()
        };
        let mut query = form_urlencoded::Serializer::new(String::new());
        query.extend_pairs([
            ("client_id", config.client.as_ref()),
            ("response_type", "code"),
            ("redirect_uri", redirect.as_str()),
            ("scope", config.scope),
            ("state", &state),
        ]);
        if config.pkce {
            let challenge =
                URL_SAFE_NO_PAD.encode(digest::digest(&digest::SHA256, verifier.as_bytes()));
            query.extend_pairs([
                ("code_challenge", challenge.as_str()),
                ("code_challenge_method", "S256"),
            ]);
        }
        if name == "codex" {
            query.extend_pairs([
                ("prompt", "login"),
                ("id_token_add_organizations", "true"),
                ("codex_cli_simplified_flow", "true"),
            ]);
        } else if name == "claude" {
            query.append_pair("code", "true");
        } else {
            query.extend_pairs([("access_type", "offline"), ("prompt", "consent")]);
        }
        let authorization_url = format!("{}?{}", config.authorize, query.finish());
        let mut pending = self.logins.lock().unwrap();
        pending.retain(|_, login| login.created.elapsed() < LIFETIME);
        if pending.len() >= 32 {
            return Err("Too many pending logins; wait for an existing login to expire".into());
        }
        pending.insert(
            id.clone(),
            PendingLogin {
                provider: name.into(),
                backend: backend.into(),
                verifier,
                state,
                redirect: redirect.clone(),
                created: Instant::now(),
            },
        );
        let mut results = self.login_results.lock().unwrap();
        results.retain(|_, (created, _)| created.elapsed() < LIFETIME);
        results.insert(id.clone(), (Instant::now(), json!({"status":"waiting"})));
        Ok(OAuthLogin {
            id,
            authorization_url,
            redirect_uri: redirect,
            expires_in: LIFETIME.as_secs(),
        })
    }

    pub fn login_backend(&self, id: &str) -> Result<(String, String), String> {
        let pending = self.logins.lock().unwrap();
        let login = pending
            .get(id)
            .filter(|p| p.created.elapsed() < LIFETIME)
            .ok_or("Login expired or already submitted; start a new login")?;
        Ok((login.provider.clone(), login.backend.clone()))
    }

    pub fn login_id_for_state(&self, provider: &str, state: &str) -> Option<String> {
        self.logins
            .lock()
            .unwrap()
            .iter()
            .find(|(_, p)| {
                p.provider == provider && p.state == state && p.created.elapsed() < LIFETIME
            })
            .map(|(id, _)| id.clone())
    }

    pub fn login_status(&self, id: &str) -> Value {
        self.login_results
            .lock()
            .unwrap()
            .get(id)
            .filter(|(created, _)| created.elapsed() < LIFETIME)
            .map(|(_, result)| result.clone())
            .unwrap_or_else(
                || json!({"status":"expired","error":"Login expired; start a new login"}),
            )
    }

    pub async fn cancel_login(&self, id: &str) {
        let _guard = self.mutation.lock().await;
        self.logins.lock().unwrap().remove(id);
        let mut results = self.login_results.lock().unwrap();
        if let Some((_, value)) = results.get_mut(id) {
            if value["status"] != "success" {
                *value = json!({"status":"cancelled"});
            }
        }
    }

    fn set_login_status(&self, id: &str, value: Value) {
        if let Some((_, result)) = self.login_results.lock().unwrap().get_mut(id) {
            if result["status"] != "cancelled" {
                *result = value;
            }
        }
    }

    pub async fn finish_login(&self, id: &str, callback: &str) -> Result<AccountSummary, String> {
        let (login, code) = {
            let mut pending = self.logins.lock().unwrap();
            let login = pending
                .get(id)
                .filter(|p| p.created.elapsed() < LIFETIME)
                .ok_or("Login expired or already submitted; start a new login")?;
            let code = match callback_code(callback, &login.redirect, &login.state) {
                Ok(code) => code,
                Err(message) => {
                    if message.starts_with("Authorization was denied") {
                        pending.remove(id);
                        self.set_login_status(id, json!({"status":"error","error":message}));
                    }
                    return Err(message);
                }
            };
            // Consume before awaiting the provider: one code may only be exchanged once.
            (pending.remove(id).unwrap(), code)
        };
        self.set_login_status(id, json!({"status":"exchanging"}));
        let result = self.exchange_login(id, login, code).await;
        if let Err(message) = &result {
            self.set_login_status(id, json!({"status":"error","error":message}));
        }
        result
    }

    async fn exchange_login(
        &self,
        id: &str,
        login: PendingLogin,
        code: String,
    ) -> Result<AccountSummary, String> {
        let config = provider(&login.provider)?;
        let (body, content_type) = if matches!(login.provider.as_str(), "codex" | "antigravity") {
            let mut body = form_urlencoded::Serializer::new(String::new());
            body.extend_pairs([
                ("grant_type", "authorization_code"),
                ("client_id", config.client.as_ref()),
                ("code", code.as_str()),
                ("redirect_uri", login.redirect.as_str()),
            ]);
            if config.pkce {
                body.append_pair("code_verifier", &login.verifier);
            }
            if let Some(secret) = &config.client_secret {
                body.append_pair("client_secret", secret.as_ref());
            }
            (body.finish(), "application/x-www-form-urlencoded")
        } else {
            (
                json!({"grant_type":"authorization_code","client_id":config.client.as_ref(),"code":code,
                "redirect_uri":login.redirect,"code_verifier":login.verifier,"state":login.state})
                .to_string(),
                "application/json",
            )
        };
        #[cfg(test)]
        let endpoint = self.refresh_endpoint.as_deref().unwrap_or(config.token);
        #[cfg(not(test))]
        let endpoint = config.token;
        let tokens = exchange(endpoint, body, content_type).await?;
        let mut account = OAuthAccount {
            id: format!("{}-{id}", login.provider),
            backend: login.backend,
            document: json!({"type":login.provider}),
            models: Vec::new(),
            revision: 0,
        };
        merge_refresh(&mut account, tokens.clone())?;
        if login.provider == "antigravity" {
            let (email, project_id) = self.antigravity_identity(account.access_token()).await?;
            account.document["email"] = json!(email);
            account.document["project_id"] = json!(project_id);
        }
        // ID-token claims are display/account-routing metadata, never management authorization.
        if let Some(claims) = tokens["id_token"]
            .as_str()
            .and_then(|jwt| jwt.split('.').nth(1))
            .and_then(|payload| URL_SAFE_NO_PAD.decode(payload).ok())
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        {
            if let Some(email) = claims["email"].as_str() {
                account.document["email"] = json!(email);
            }
            if let Some(id) = claims["https://api.openai.com/auth"]["chatgpt_account_id"].as_str() {
                account.document["account_id"] = json!(id);
            }
        }
        if let Some(email) = tokens["account"]["email_address"].as_str() {
            account.document["email"] = json!(email);
        }
        for key in ["account", "organization"] {
            if tokens[key].is_object() {
                account.document[key] = tokens[key].clone();
            }
        }
        let _guard = self.mutation.lock().await;
        if self.login_status(id)["status"] == "cancelled" {
            return Err("Login was cancelled".into());
        }
        let summary = self.save_locked(account)?;
        self.set_login_status(id, json!({"status":"success","account":summary}));
        Ok(summary)
    }
}

impl LlmAccounts {
    async fn antigravity_identity(&self, access_token: &str) -> Result<(String, String), String> {
        #[cfg(test)]
        let base = self
            .refresh_endpoint
            .as_deref()
            .and_then(|endpoint| endpoint.strip_suffix("/token"))
            .unwrap_or("");
        #[cfg(test)]
        let userinfo_url = format!("{base}/userinfo");
        #[cfg(not(test))]
        let userinfo_url = "https://www.googleapis.com/oauth2/v2/userinfo?alt=json".to_string();
        let profile = antigravity_request(&userinfo_url, "GET", access_token, None, false).await?;
        let email = profile["email"]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or("Antigravity userinfo response has no email")?
            .to_string();

        #[cfg(test)]
        let load_url = format!("{base}/v1internal:loadCodeAssist");
        #[cfg(not(test))]
        let load_url =
            "https://daily-cloudcode-pa.googleapis.com/v1internal:loadCodeAssist".to_string();
        let load = antigravity_request(
            &load_url,
            "POST",
            access_token,
            Some(json!({"metadata":{"ideType":"ANTIGRAVITY"}})),
            false,
        )
        .await?;
        if let Some(project) = antigravity_project(&load) {
            return Ok((email, project));
        }

        let tier = load["allowedTiers"]
            .as_array()
            .and_then(|tiers| {
                tiers.iter().find_map(|tier| {
                    tier["isDefault"]
                        .as_bool()
                        .unwrap_or(false)
                        .then(|| tier["id"].as_str())
                        .flatten()
                })
            })
            .or_else(|| load["currentTier"]["id"].as_str())
            .unwrap_or("free-tier");
        #[cfg(test)]
        let onboard_url = format!("{base}/v1internal:onboardUser");
        #[cfg(not(test))]
        let onboard_url =
            "https://daily-cloudcode-pa.googleapis.com/v1internal:onboardUser".to_string();
        let body = json!({
            "tier_id":tier,
            "metadata":{"ide_type":"ANTIGRAVITY","ide_version":"2.9.1","ide_name":"antigravity"}
        });
        for attempt in 0..5 {
            let response =
                antigravity_request(&onboard_url, "POST", access_token, Some(body.clone()), true)
                    .await?;
            if response["done"].as_bool().unwrap_or(false) {
                if let Some(project) = antigravity_project(&response["response"]) {
                    return Ok((email, project));
                }
                return Err("Antigravity onboarding response has no project_id".into());
            }
            if attempt < 4 {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        Err("Antigravity onboarding did not complete after 5 attempts".into())
    }
}

fn antigravity_project(value: &Value) -> Option<String> {
    for key in ["cloudaicompanionProject", "projectId", "project"] {
        if let Some(project) = value[key].as_str().map(str::trim).filter(|v| !v.is_empty()) {
            return Some(project.to_string());
        }
        if let Some(project) = value[key]["id"]
            .as_str()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Some(project.to_string());
        }
    }
    None
}

async fn antigravity_request(
    endpoint: &str,
    method: &str,
    access_token: &str,
    body: Option<Value>,
    onboard: bool,
) -> Result<Value, String> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots();
    #[cfg(test)]
    let connector = connector.https_or_http();
    #[cfg(not(test))]
    let connector = connector.https_only();
    let client = hyper::Client::builder().build::<_, hyper::Body>(connector.enable_http1().build());
    let user_agent = if onboard {
        "antigravity/hub/2.9.1 darwin/arm64 google-api-nodejs-client/10.3.0"
    } else {
        "antigravity/hub/2.9.1 darwin/arm64"
    };
    let mut request = hyper::Request::builder()
        .method(method)
        .uri(endpoint)
        .header("authorization", format!("Bearer {access_token}"))
        .header("accept", "*/*")
        .header("user-agent", user_agent);
    if body.is_some() {
        request = request.header("content-type", "application/json");
    }
    if onboard {
        request = request.header("x-goog-api-client", "gl-node/22.21.1");
    }
    let request = request
        .body(hyper::Body::from(
            body.map(|value| value.to_string()).unwrap_or_default(),
        ))
        .map_err(|_| "Cannot build Antigravity identity request")?;
    let fetch = async {
        use hyper::body::HttpBody;
        let mut response = client
            .request(request)
            .await
            .map_err(|_| "Antigravity identity request failed".to_string())?;
        if !response.status().is_success() {
            return Err(format!(
                "Antigravity identity request returned HTTP {}",
                response.status().as_u16()
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.body_mut().data().await {
            let chunk = chunk.map_err(|_| "Cannot read Antigravity identity response")?;
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err("Antigravity identity response is too large".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| "Invalid Antigravity identity response".into())
    };
    tokio::time::timeout(Duration::from_secs(30), fetch)
        .await
        .map_err(|_| "Antigravity identity request timed out".to_string())?
}

fn callback_code(callback: &str, redirect: &str, state: &str) -> Result<String, String> {
    let (base, query) = callback
        .trim()
        .split_once('?')
        .ok_or("Paste the full callback URL including code and state")?;
    if base != redirect || query.contains('#') {
        return Err("Callback URL does not match this provider".into());
    }
    let mut values = std::collections::BTreeMap::new();
    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        if values
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            return Err("Duplicate callback parameter".into());
        }
    }
    if values.get("state").map(String::as_str) != Some(state) {
        return Err("OAuth state mismatch; use the link from this login".into());
    }
    if values.contains_key("error") {
        return Err("Authorization was denied; start a new login".into());
    }
    values
        .remove("code")
        .filter(|s| !s.is_empty() && s.len() <= 8192)
        .ok_or_else(|| "Callback URL has no valid authorization code".into())
}

async fn exchange(endpoint: &str, body: String, content_type: &str) -> Result<Value, String> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots();
    #[cfg(test)]
    let connector = connector.https_or_http();
    #[cfg(not(test))]
    let connector = connector.https_only();
    let client = hyper::Client::builder().build::<_, hyper::Body>(connector.enable_http1().build());
    let request = hyper::Request::post(endpoint)
        .header("content-type", content_type)
        .header("accept", "application/json")
        .body(hyper::Body::from(body))
        .map_err(|_| "Cannot build token exchange")?;
    let fetch = async {
        use hyper::body::HttpBody;
        let mut response = client
            .request(request)
            .await
            .map_err(|_| "OAuth token exchange transport failed; start a new login")?;
        if !response.status().is_success() {
            return Err(format!(
                "OAuth token exchange rejected with HTTP {}; start a new login",
                response.status().as_u16()
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.body_mut().data().await {
            let chunk = chunk.map_err(|_| "Cannot read token exchange response")?;
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err("Token exchange response is too large".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| "Invalid token exchange response".into())
    };
    tokio::time::timeout(Duration::from_secs(30), fetch)
        .await
        .map_err(|_| "OAuth token exchange timed out; start a new login")?
}
