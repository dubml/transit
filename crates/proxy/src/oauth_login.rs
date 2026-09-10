//! Browser authorization with PKCE. Credentials and verifiers never leave the server.
use super::{merge_refresh, AccountSummary, LlmAccounts, OAuthAccount};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::{
    digest,
    rand::{SecureRandom, SystemRandom},
};
use serde::Serialize;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const LIFETIME: Duration = Duration::from_secs(600);

struct Provider {
    authorize: &'static str,
    token: &'static str,
    client: &'static str,
    redirect: &'static str,
    scope: &'static str,
}

fn provider(name: &str) -> Result<Provider, String> {
    match name {
        "codex" => Ok(Provider {
            authorize: "https://auth.openai.com/oauth/authorize",
            token: "https://auth.openai.com/oauth/token",
            client: "app_EMoamEEZ73f0CkXaXp7hrann",
            redirect: "http://localhost:1455/auth/callback",
            scope: "openid email profile offline_access",
        }),
        "claude" => Ok(Provider {
            authorize: "https://claude.ai/oauth/authorize",
            token: "https://platform.claude.com/v1/oauth/token",
            client: "9d1c250a-e61b-44d9-88ed-5944d1962f5e",
            redirect: "http://localhost:54545/callback",
            scope: "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload",
        }),
        _ => Err("Unsupported OAuth provider".into()),
    }
}

pub(super) struct PendingLogin {
    provider: String,
    backend: String,
    verifier: String,
    state: String,
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
        if !self.enabled() {
            return Err("OAuth management is not configured".into());
        }
        let config = provider(name)?;
        let id = random()?;
        let state = random()?;
        let verifier = random()?;
        let challenge =
            URL_SAFE_NO_PAD.encode(digest::digest(&digest::SHA256, verifier.as_bytes()));
        let mut query = form_urlencoded::Serializer::new(String::new());
        query.extend_pairs([
            ("client_id", config.client),
            ("response_type", "code"),
            ("redirect_uri", config.redirect),
            ("scope", config.scope),
            ("state", &state),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
        ]);
        if name == "codex" {
            query.extend_pairs([
                ("prompt", "login"),
                ("id_token_add_organizations", "true"),
                ("codex_cli_simplified_flow", "true"),
            ]);
        } else {
            query.append_pair("code", "true");
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
                created: Instant::now(),
            },
        );
        let mut results = self.login_results.lock().unwrap();
        results.retain(|_, (created, _)| created.elapsed() < LIFETIME);
        results.insert(id.clone(), (Instant::now(), json!({"status":"waiting"})));
        Ok(OAuthLogin {
            id,
            authorization_url,
            redirect_uri: config.redirect.into(),
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
            let code = match callback_code(callback, &provider(&login.provider)?, &login.state) {
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
        let (body, content_type) = if login.provider == "codex" {
            (
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs([
                        ("grant_type", "authorization_code"),
                        ("client_id", config.client),
                        ("code", code.as_str()),
                        ("redirect_uri", config.redirect),
                        ("code_verifier", login.verifier.as_str()),
                    ])
                    .finish(),
                "application/x-www-form-urlencoded",
            )
        } else {
            (
                json!({"grant_type":"authorization_code","client_id":config.client,"code":code,
                "redirect_uri":config.redirect,"code_verifier":login.verifier,"state":login.state})
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

fn callback_code(callback: &str, config: &Provider, state: &str) -> Result<String, String> {
    let (base, query) = callback
        .trim()
        .split_once('?')
        .ok_or("Paste the full callback URL including code and state")?;
    if base != config.redirect || query.contains('#') {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn login_pkce_exchange_saves_credentials_and_rejects_replay() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let capture = calls.clone();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let claims = URL_SAFE_NO_PAD.encode(json!({"email":"login@example.test","https://api.openai.com/auth":{"chatgpt_account_id":"account-123"}}).to_string());
        let token_body = json!({"access_token":"private-access","refresh_token":"private-refresh","expires_in":3600,"id_token":format!("header.{claims}.signature")}).to_string();
        let service = hyper::service::make_service_fn(move |_| {
            let capture = capture.clone();
            let body = token_body.clone();
            async move {
                Ok::<_, std::convert::Infallible>(hyper::service::service_fn(
                    move |req: hyper::Request<hyper::Body>| {
                        let capture = capture.clone();
                        let body = body.clone();
                        async move {
                            let content_type =
                                req.headers()["content-type"].to_str().unwrap().to_string();
                            let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                            capture
                                .lock()
                                .unwrap()
                                .push((content_type, String::from_utf8(bytes.to_vec()).unwrap()));
                            Ok::<_, std::convert::Infallible>(hyper::Response::new(
                                hyper::Body::from(body),
                            ))
                        }
                    },
                ))
            }
        });
        let server = tokio::spawn(hyper::Server::from_tcp(listener).unwrap().serve(service));
        for (index, name) in ["codex", "claude"].into_iter().enumerate() {
            let directory = Directory(
                std::env::temp_dir().join(format!("transit-oauth-login-{}", random().unwrap())),
            );
            let store = LlmAccounts {
                refresh_endpoint: Some(format!("http://{addr}/token")),
                ..Default::default()
            };
            store
                .configure(&directory.0, "test-admin-token-long-enough".into())
                .unwrap();
            let login = store.start_login(name, "").unwrap();
            let query: std::collections::BTreeMap<_, _> = form_urlencoded::parse(
                login
                    .authorization_url
                    .split_once('?')
                    .unwrap()
                    .1
                    .as_bytes(),
            )
            .into_owned()
            .collect();
            assert_eq!(query["code_challenge_method"], "S256");
            assert!(!serde_json::to_string(&login).unwrap().contains("verifier"));
            let callback = format!(
                "{}?code=auth%2Bcode%26value&state={}",
                login.redirect_uri, query["state"]
            );
            assert!(store
                .finish_login(
                    &login.id,
                    &format!("{}?code=x&state=wrong", login.redirect_uri)
                )
                .await
                .unwrap_err()
                .contains("state mismatch"));
            let summary = store.finish_login(&login.id, &callback).await.unwrap();
            assert_eq!(store.login_status(&login.id)["status"], "success");
            assert_eq!(store.login_status(&login.id)["account"]["id"], summary.id);
            let cancelled = store.start_login(name, "subscription").unwrap();
            store.cancel_login(&cancelled.id).await;
            assert_eq!(store.login_status(&cancelled.id)["status"], "cancelled");
            assert!(store.login_backend(&cancelled.id).is_err());
            assert_eq!(summary.email, "login@example.test");
            assert!(!serde_json::to_string(&summary)
                .unwrap()
                .contains("private-access"));
            let account = store.get(&summary.id).unwrap();
            assert!(account.backend.is_empty());
            assert_eq!(account.document["access_token"], "private-access");
            assert_eq!(account.document["refresh_token"], "private-refresh");
            assert_eq!(account.document["account_id"], "account-123");
            let restored = LlmAccounts::default();
            restored
                .configure(&directory.0, "test-admin-token-long-enough".into())
                .unwrap();
            assert_eq!(
                restored.get(&summary.id).unwrap().document,
                account.document
            );
            assert!(store
                .finish_login(&login.id, &callback)
                .await
                .unwrap_err()
                .contains("already submitted"));
            let calls = calls.lock().unwrap();
            let body: Value = if name == "codex" {
                assert_eq!(calls[index].0, "application/x-www-form-urlencoded");
                let map: std::collections::BTreeMap<_, _> =
                    form_urlencoded::parse(calls[index].1.as_bytes())
                        .into_owned()
                        .collect();
                json!(map)
            } else {
                assert_eq!(calls[index].0, "application/json");
                serde_json::from_str(&calls[index].1).unwrap()
            };
            assert_eq!(body["code"], "auth+code&value");
            assert_eq!(body["grant_type"], "authorization_code");
            assert_eq!(body["redirect_uri"], login.redirect_uri);
            assert_eq!(
                query["code_challenge"],
                URL_SAFE_NO_PAD.encode(digest::digest(
                    &digest::SHA256,
                    body["code_verifier"].as_str().unwrap().as_bytes()
                ))
            );
        }
        server.abort();
    }

    #[test]
    fn callback_rejects_wrong_origin_duplicates_denial_and_expired_sessions() {
        let config = provider("codex").unwrap();
        for callback in [
            "https://evil.test/?code=a&state=s",
            "http://localhost:1455/auth/callback?code=a&state=s&state=s",
            "http://localhost:1455/auth/callback?error=denied&state=s",
            "http://localhost:1455/auth/callback?state=s",
        ] {
            assert!(callback_code(callback, &config, "s").is_err());
        }
        let directory = Directory(
            std::env::temp_dir().join(format!("transit-oauth-login-{}", random().unwrap())),
        );
        let store = LlmAccounts::default();
        assert!(store.start_login("codex", "backend").is_err());
        store
            .configure(&directory.0, "test-admin-token-long-enough".into())
            .unwrap();
        let login = store.start_login("codex", "backend").unwrap();
        store
            .logins
            .lock()
            .unwrap()
            .get_mut(&login.id)
            .unwrap()
            .created = Instant::now() - LIFETIME;
        assert!(store.login_backend(&login.id).is_err());
        for _ in 0..32 {
            store.start_login("codex", "backend").unwrap();
        }
        assert!(store
            .start_login("codex", "backend")
            .unwrap_err()
            .contains("Too many"));
    }
}
