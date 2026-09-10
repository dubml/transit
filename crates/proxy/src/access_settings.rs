use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{
    digest, pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::{self, Write},
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TlsSettings {
    pub enable: bool,
    pub cert: String,
    pub key: String,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RemoteManagement {
    pub allow_remote: bool,
    pub disable_control_panel: bool,
    pub disable_auto_update_panel: bool,
    pub panel_github_repository: String,
    pub secret_key: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccessConfig {
    pub host: String,
    pub port: u16,
    pub auth_dir: String,
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default)]
    pub tls: TlsSettings,
    #[serde(default)]
    pub remote_management: RemoteManagement,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedConfig {
    revision: u64,
    config: AccessConfig,
}

struct Runtime {
    path: PathBuf,
    writable: bool,
    saved: SavedConfig,
    active: AccessConfig,
    local_token: String,
}

#[derive(Default)]
pub struct AccessSettings {
    data: RwLock<Option<Runtime>>,
    verified: Mutex<Option<Vec<u8>>>,
}

#[derive(Serialize)]
pub struct AccessView {
    pub writable: bool,
    pub revision: u64,
    pub config: AccessConfig,
    pub active: ActiveSettings,
    pub restart_required: bool,
    pub secret_configured: bool,
    pub config_file: String,
}

#[derive(Serialize)]
pub struct ActiveSettings {
    pub host: String,
    pub port: u16,
    pub auth_dir: String,
    pub tls: bool,
}

#[derive(Debug)]
pub enum SaveError {
    Conflict,
    Invalid(String),
    Io(io::Error),
}

impl AccessSettings {
    pub fn configure(&self, path: PathBuf, defaults: AccessConfig) -> io::Result<AccessConfig> {
        let mut saved = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice::<SavedConfig>(&bytes)
                .map_err(|_| io::Error::other("Invalid access configuration JSON"))?,
            Err(err) if err.kind() == io::ErrorKind::NotFound => SavedConfig {
                revision: 0,
                config: defaults,
            },
            Err(err) => return Err(err),
        };
        validate(&mut saved.config).map_err(io::Error::other)?;
        validate_storage_location(&path, &saved.config.auth_dir).map_err(io::Error::other)?;
        let hash = &mut saved.config.remote_management.secret_key;
        if !hash.is_empty() && !hash.starts_with("pbkdf2$") {
            *hash = hash_secret(hash).map_err(io::Error::other)?;
            if path.exists() {
                write_private(&path, &serde_json::to_vec_pretty(&saved)?)?;
            }
        }
        let mut random = [0; 32];
        SystemRandom::new()
            .fill(&mut random)
            .map_err(|_| io::Error::other("Could not create management session"))?;
        let active = saved.config.clone();
        *self.data.write().unwrap() = Some(Runtime {
            path,
            writable: true,
            saved,
            active: active.clone(),
            local_token: STANDARD.encode(random),
        });
        *self.verified.lock().unwrap() = None;
        Ok(active)
    }

    /// Deployment-managed settings are read once and never rewritten by a pod.
    pub fn configure_read_only(
        &self,
        path: Option<&std::path::Path>,
        defaults: AccessConfig,
    ) -> io::Result<AccessConfig> {
        let mut saved = match path {
            Some(path) => serde_json::from_slice::<SavedConfig>(&std::fs::read(path)?)
                .map_err(|_| io::Error::other("Invalid access configuration JSON"))?,
            None => SavedConfig { revision: 0, config: defaults },
        };
        validate(&mut saved.config).map_err(io::Error::other)?;
        let secret = &mut saved.config.remote_management.secret_key;
        if !secret.is_empty() && !secret.starts_with("pbkdf2$") {
            *secret = hash_secret(secret).map_err(io::Error::other)?;
        }
        let mut random = [0; 32];
        SystemRandom::new().fill(&mut random)
            .map_err(|_| io::Error::other("Could not create management session"))?;
        let active = saved.config.clone();
        *self.data.write().unwrap() = Some(Runtime {
            path: path.map(std::path::Path::to_path_buf).unwrap_or_default(),
            writable: false,
            saved,
            active: active.clone(),
            local_token: STANDARD.encode(random),
        });
        *self.verified.lock().unwrap() = None;
        Ok(active)
    }

    pub fn configured(&self) -> bool {
        self.data.read().unwrap().is_some()
    }

    pub fn view(&self) -> Option<AccessView> {
        self.data.read().unwrap().as_ref().map(|runtime| {
            let mut config = runtime.saved.config.clone();
            let secret_configured = !config.remote_management.secret_key.is_empty();
            config.remote_management.secret_key.clear();
            AccessView {
                writable: runtime.writable,
                revision: runtime.saved.revision,
                restart_required: config.host != runtime.active.host
                    || config.port != runtime.active.port
                    || config.auth_dir != runtime.active.auth_dir
                    || config.tls != runtime.active.tls,
                active: ActiveSettings {
                    host: runtime.active.host.clone(),
                    port: runtime.active.port,
                    auth_dir: runtime.active.auth_dir.clone(),
                    tls: runtime.active.tls.enable,
                },
                config,
                secret_configured,
                config_file: runtime.path.display().to_string(),
            }
        })
    }

    pub fn remote(&self) -> Option<RemoteManagement> {
        self.data
            .read()
            .unwrap()
            .as_ref()
            .map(|r| r.saved.config.remote_management.clone())
    }

    pub fn active_tls(&self) -> TlsSettings {
        self.data
            .read()
            .unwrap()
            .as_ref()
            .map(|r| r.active.tls.clone())
            .unwrap_or_default()
    }

    pub fn local_token(&self) -> Option<String> {
        self.data
            .read()
            .unwrap()
            .as_ref()
            .filter(|r| r.saved.config.remote_management.secret_key.is_empty())
            .map(|r| r.local_token.clone())
    }

    pub fn authorized(&self, candidate: &str) -> bool {
        if candidate.is_empty() || candidate.len() > 4096 {
            return false;
        }
        let data = self.data.read().unwrap();
        let Some(runtime) = data.as_ref() else {
            return false;
        };
        let hash = &runtime.saved.config.remote_management.secret_key;
        if hash.is_empty() {
            return constant_eq(&runtime.local_token, candidate);
        }
        let candidate_digest = digest::digest(&digest::SHA256, candidate.as_bytes())
            .as_ref()
            .to_vec();
        let mut verified = self.verified.lock().unwrap();
        if verified.as_ref().is_some_and(|v| v == &candidate_digest) {
            return true;
        }
        if verify_secret(hash, candidate) {
            *verified = Some(candidate_digest);
            return true;
        }
        false
    }

    pub fn save(
        &self,
        revision: u64,
        mut config: AccessConfig,
        secret: Option<String>,
    ) -> Result<(), SaveError> {
        let mut data = self.data.write().unwrap();
        let runtime = data
            .as_mut()
            .ok_or_else(|| SaveError::Invalid("Access configuration is unavailable".into()))?;
        if !runtime.writable {
            return Err(SaveError::Invalid("Access settings are managed by the Kubernetes deployment".into()));
        }
        if revision != runtime.saved.revision {
            return Err(SaveError::Conflict);
        }
        match std::fs::read(&runtime.path) {
            Ok(bytes) => {
                let on_disk: SavedConfig =
                    serde_json::from_slice(&bytes).map_err(|_| SaveError::Conflict)?;
                if on_disk.revision != revision || on_disk.config != runtime.saved.config {
                    return Err(SaveError::Conflict);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && revision == 0 => {}
            Err(error) => return Err(SaveError::Io(error)),
        }
        // An omitted password preserves the hash. Empty explicitly clears it.
        config.remote_management.secret_key = match secret {
            None => runtime.saved.config.remote_management.secret_key.clone(),
            Some(value) if value.is_empty() => String::new(),
            Some(value) => hash_secret(&value).map_err(SaveError::Invalid)?,
        };
        validate(&mut config).map_err(SaveError::Invalid)?;
        validate_storage_location(&runtime.path, &config.auth_dir).map_err(SaveError::Invalid)?;
        if config.auth_dir != runtime.saved.config.auth_dir {
            let directory =
                expand_path(&config.auth_dir).map_err(|e| SaveError::Invalid(e.to_string()))?;
            if directory.is_file() {
                return Err(SaveError::Invalid(
                    "Authentication directory must be a directory".into(),
                ));
            }
            // Validate an existing destination with the actual account loader; never move credentials.
            if directory.exists() {
                crate::LlmAccounts::default().configure_local(&directory).map_err(|_| SaveError::Invalid("Authentication directory contains invalid account files or is not writable".into()))?;
            }
        }
        let saved = SavedConfig {
            revision: revision
                .checked_add(1)
                .ok_or_else(|| SaveError::Invalid("Configuration revision exhausted".into()))?,
            config,
        };
        let bytes = serde_json::to_vec_pretty(&saved).map_err(|e| SaveError::Io(e.into()))?;
        let next_local_token = if saved.config.remote_management.secret_key
            != runtime.saved.config.remote_management.secret_key
        {
            let mut random = [0; 32];
            SystemRandom::new().fill(&mut random).map_err(|_| {
                SaveError::Io(io::Error::other(
                    "Could not rotate local management session",
                ))
            })?;
            Some(STANDARD.encode(random))
        } else {
            None
        };
        write_private(&runtime.path, &bytes).map_err(SaveError::Io)?;
        runtime.saved = saved;
        if let Some(token) = next_local_token {
            runtime.local_token = token;
        }
        *self.verified.lock().unwrap() = None;
        Ok(())
    }

    pub fn authenticate_api(&self, headers: &mut HeaderMap, uri: &mut http::Uri) -> bool {
        let data = self.data.read().unwrap();
        let Some(runtime) = data.as_ref() else {
            return true;
        };
        let keys = &runtime.saved.config.api_keys;
        if keys.is_empty() {
            return true;
        }
        let matches = |candidate: &str| keys.iter().any(|key| constant_eq(key, candidate));
        let mut accepted = false;
        for name in [
            header::AUTHORIZATION.as_str(),
            "x-api-key",
            "x-goog-api-key",
        ] {
            let candidate = headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let candidate = if name == "authorization" {
                bearer(candidate)
            } else {
                candidate
            };
            if matches(candidate) {
                accepted = true;
                headers.remove(name);
            }
        }
        if let Some(query) = uri.query() {
            let mut remaining = form_urlencoded::Serializer::new(String::new());
            let mut query_removed = false;
            for (name, value) in form_urlencoded::parse(query.as_bytes()) {
                if (name == "key" || name == "auth_token") && matches(&value) {
                    accepted = true;
                    query_removed = true;
                } else {
                    remaining.append_pair(&name, &value);
                }
            }
            if query_removed {
                let query = remaining.finish();
                let path = if query.is_empty() {
                    uri.path().to_string()
                } else {
                    format!("{}?{query}", uri.path())
                };
                let mut parts = uri.clone().into_parts();
                if let Ok(path) = path.parse() {
                    parts.path_and_query = Some(path);
                    if let Ok(next) = http::Uri::from_parts(parts) {
                        *uri = next;
                    }
                }
            }
        }
        accepted
    }
}

pub fn bearer(value: &str) -> &str {
    value
        .split_once(' ')
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
        .map(|(_, token)| token)
        .unwrap_or(value)
}

fn constant_eq(expected: &str, candidate: &str) -> bool {
    let mut diff = expected.len() ^ candidate.len();
    for (i, b) in expected.bytes().enumerate() {
        diff |= (b ^ candidate.as_bytes().get(i).copied().unwrap_or(0)) as usize;
    }
    diff == 0
}

fn hash_secret(secret: &str) -> Result<String, String> {
    if secret.len() < 24 || secret.len() > 4096 || secret.trim() != secret {
        return Err(
            "Management key must contain 24–4096 characters without surrounding whitespace".into(),
        );
    }
    let mut salt = [0; 16];
    SystemRandom::new()
        .fill(&mut salt)
        .map_err(|_| "Could not secure management key")?;
    let mut hash = [0; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(600_000).unwrap(),
        &salt,
        secret.as_bytes(),
        &mut hash,
    );
    Ok(format!(
        "pbkdf2${}${}",
        STANDARD.encode(salt),
        STANDARD.encode(hash)
    ))
}

fn verify_secret(hash: &str, candidate: &str) -> bool {
    let parts: Vec<_> = hash.split('$').collect();
    if parts.len() != 3 || parts[0] != "pbkdf2" {
        return false;
    }
    let (Ok(salt), Ok(hash)) = (STANDARD.decode(parts[1]), STANDARD.decode(parts[2])) else {
        return false;
    };
    salt.len() == 16
        && hash.len() == 32
        && pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(600_000).unwrap(),
            &salt,
            candidate.as_bytes(),
            &hash,
        )
        .is_ok()
}

fn validate(config: &mut AccessConfig) -> Result<(), String> {
    config.host = config.host.trim().to_string();
    if config.host == "localhost" {
        config.host = "127.0.0.1".into();
    }
    if config.host.is_empty() {
        config.host = "0.0.0.0".into();
    }
    config
        .host
        .parse::<IpAddr>()
        .map_err(|_| "Host must be an IPv4/IPv6 address or localhost")?;
    if config.port == 0 {
        return Err("Port must be between 1 and 65535".into());
    }
    expand_path(&config.auth_dir).map_err(|e| e.to_string())?;
    let mut seen = BTreeSet::new();
    if config.api_keys.len() > 100 {
        return Err("At most 100 API keys are supported".into());
    }
    for key in &config.api_keys {
        if key.len() < 8
            || key.len() > 4096
            || key.trim() != key
            || key.chars().any(char::is_control)
        {
            return Err("API keys must contain 8–4096 characters without surrounding whitespace or control characters".into());
        }
        if !seen.insert(key) {
            return Err("Duplicate API keys are not allowed".into());
        }
    }
    let remote = &mut config.remote_management;
    if remote.secret_key.starts_with("pbkdf2$") {
        let parts: Vec<_> = remote.secret_key.split('$').collect();
        if parts.len() != 3
            || !STANDARD.decode(parts[1]).is_ok_and(|salt| salt.len() == 16)
            || !STANDARD.decode(parts[2]).is_ok_and(|hash| hash.len() == 32)
        {
            return Err("Invalid management key hash".into());
        }
    }
    remote.panel_github_repository = remote
        .panel_github_repository
        .trim()
        .trim_end_matches('/')
        .to_string();
    if !remote.panel_github_repository.is_empty() {
        github_release_url(&remote.panel_github_repository)?;
    }
    if remote.allow_remote && remote.secret_key.is_empty() {
        return Err("Set a management key before allowing remote access".into());
    }
    tls_config(&config.tls).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn github_release_url(repository: &str) -> Result<String, String> {
    let path = repository
        .strip_prefix("https://github.com/")
        .or_else(|| {
            repository
                .strip_prefix("https://api.github.com/repos/")
                .and_then(|s| s.strip_suffix("/releases/latest"))
        })
        .ok_or("Panel repository must be a GitHub repository URL or latest-release API URL")?;
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|s| {
            s.is_empty()
                || *s == "."
                || *s == ".."
                || !s
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        })
    {
        return Err("Invalid GitHub owner/repository".into());
    }
    Ok(format!(
        "https://api.github.com/repos/{path}/releases/latest"
    ))
}

fn validate_storage_location(settings: &Path, auth_dir: &str) -> Result<(), String> {
    let directory = expand_path(auth_dir).map_err(|error| error.to_string())?;
    let parent = settings
        .parent()
        .ok_or("Access configuration file has no parent")?;
    if std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf())
        == std::fs::canonicalize(&directory).unwrap_or(directory)
    {
        return Err(
            "Keep the access configuration file outside the OAuth authentication directory".into(),
        );
    }
    Ok(())
}

pub fn expand_path(value: &str) -> io::Result<PathBuf> {
    let value = value.trim();
    if value.is_empty() {
        return Err(io::Error::other("Path must not be empty"));
    }
    if value == "~" || value.starts_with("~/") {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| io::Error::other("HOME is unavailable; use an absolute path"))?;
        return Ok(PathBuf::from(home).join(value.strip_prefix("~/").unwrap_or("")));
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(io::Error::other("Use an absolute path or ~/ path"));
    }
    Ok(path)
}

pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Configuration path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut random = [0; 12];
    SystemRandom::new()
        .fill(&mut random)
        .map_err(|_| io::Error::other("Could not create temporary file"))?;
    let temporary = parent.join(format!(
        ".access-{}.tmp",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random)
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        let _ = std::fs::File::open(parent).and_then(|dir| dir.sync_all());
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub fn tls_config(settings: &TlsSettings) -> io::Result<Option<Arc<rustls::ServerConfig>>> {
    if !settings.enable {
        return Ok(None);
    }
    let cert = std::fs::read(expand_path(&settings.cert)?)
        .map_err(|_| io::Error::other("Cannot read TLS certificate"))?;
    let key = std::fs::read(expand_path(&settings.key)?)
        .map_err(|_| io::Error::other("Cannot read TLS private key"))?;
    if cert.len() > 1024 * 1024 || key.len() > 1024 * 1024 {
        return Err(io::Error::other("TLS files exceed 1 MiB"));
    }
    let certificates: Vec<_> = rustls_pemfile::certs(&mut cert.as_slice())?
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key.as_slice())?;
    if keys.is_empty() {
        keys = rustls_pemfile::rsa_private_keys(&mut key.as_slice())?;
    }
    if keys.is_empty() {
        keys = rustls_pemfile::ec_private_keys(&mut key.as_slice())?;
    }
    let key = keys
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("TLS private key is not a supported PEM key"))?;
    validate_key_pair(&certificates, &key)?;
    let mut config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, rustls::PrivateKey(key))
        .map_err(|_| io::Error::other("Invalid TLS certificate/private key pair"))?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Some(Arc::new(config)))
}

fn validate_key_pair(certificates: &[rustls::Certificate], key: &[u8]) -> io::Result<()> {
    let invalid =
        || io::Error::other("Invalid, expired or mismatched TLS certificate/private key pair");
    let certificate = certificates.first().ok_or_else(invalid)?;
    let (_, leaf) = x509_parser::parse_x509_certificate(&certificate.0).map_err(|_| invalid())?;
    if !leaf.validity().is_valid() {
        return Err(invalid());
    }
    let key = rustls::sign::any_supported_type(&rustls::PrivateKey(key.to_vec()))
        .map_err(|_| invalid())?;
    use rustls::SignatureScheme as Scheme;
    let signer = key
        .choose_scheme(&[
            Scheme::ED25519,
            Scheme::ECDSA_NISTP256_SHA256,
            Scheme::ECDSA_NISTP384_SHA384,
            Scheme::RSA_PSS_SHA256,
            Scheme::RSA_PKCS1_SHA256,
        ])
        .ok_or_else(invalid)?;
    let algorithm: &dyn ring::signature::VerificationAlgorithm = match signer.scheme() {
        Scheme::ED25519 => &ring::signature::ED25519,
        Scheme::ECDSA_NISTP256_SHA256 => &ring::signature::ECDSA_P256_SHA256_ASN1,
        Scheme::ECDSA_NISTP384_SHA384 => &ring::signature::ECDSA_P384_SHA384_ASN1,
        Scheme::RSA_PSS_SHA256 => &ring::signature::RSA_PSS_2048_8192_SHA256,
        Scheme::RSA_PKCS1_SHA256 => &ring::signature::RSA_PKCS1_2048_8192_SHA256,
        _ => return Err(invalid()),
    };
    let challenge = b"transit TLS key-pair validation";
    let signature = signer.sign(challenge).map_err(|_| invalid())?;
    ring::signature::UnparsedPublicKey::new(algorithm, &leaf.public_key().subject_public_key.data)
        .verify(challenge, &signature)
        .map_err(|_| invalid())
}

pub async fn serve_router(
    app: axum::Router,
    addr: SocketAddr,
    tls: TlsSettings,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let Some(config) = tls_config(&tls)? else {
        return axum::Server::try_bind(&addr)
            .map_err(io::Error::other)?
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(io::Error::other);
    };
    use futures_util::StreamExt;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let acceptor = tokio_rustls::TlsAcceptor::from(config);
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener)
        .map(move |stream| {
            let acceptor = acceptor.clone();
            async move {
                let stream = stream?;
                tokio::time::timeout(std::time::Duration::from_secs(10), acceptor.accept(stream))
                    .await
                    .map_err(io::Error::other)?
            }
        })
        .buffer_unordered(64)
        .filter_map(|result| async { result.ok().map(Ok::<_, io::Error>) });
    let make_service = hyper::service::make_service_fn(
        move |conn: &tokio_rustls::server::TlsStream<tokio::net::TcpStream>| {
            let app = app.clone();
            let peer = conn.get_ref().0.peer_addr();
            async move { peer.map(|peer| app.layer(axum::Extension(axum::extract::ConnectInfo(peer)))) }
        },
    );
    axum::Server::builder(hyper::server::accept::from_stream(Box::pin(incoming)))
        .serve(make_service)
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let mut nonce = [0; 12];
            SystemRandom::new().fill(&mut nonce).unwrap();
            let dir = std::env::temp_dir().join(format!(
                "transit-access-{}",
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(nonce)
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn defaults(&self) -> AccessConfig {
            AccessConfig {
                host: "127.0.0.1".into(),
                port: 8080,
                auth_dir: self.0.join("accounts").display().to_string(),
                api_keys: vec![],
                tls: Default::default(),
                remote_management: Default::default(),
            }
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn saved_settings_hash_secrets_rotate_sessions_and_apply_after_restart() {
        let dir = Directory::new();
        let settings = AccessSettings::default();
        let path = dir.0.join("access.json");
        settings.configure(path.clone(), dir.defaults()).unwrap();
        let local = settings.local_token().unwrap();
        assert!(settings.authorized(&local));
        let mut config = settings.view().unwrap().config;
        config.port = 9090;
        config.remote_management.allow_remote = true;
        let secret = "a-long-management-key-for-the-first-save";
        settings.save(0, config, Some(secret.into())).unwrap();
        let view = settings.view().unwrap();
        assert_eq!(view.active.port, 8080);
        assert!(view.restart_required);
        assert!(view.config.remote_management.secret_key.is_empty());
        assert!(settings.authorized(secret));
        assert!(!settings.authorized(&local));
        assert!(settings.local_token().is_none());
        let bytes = std::fs::read_to_string(&path).unwrap();
        assert!(!bytes.contains(secret));
        assert!(bytes.contains("pbkdf2$"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert!(matches!(
            settings.save(0, view.config.clone(), None),
            Err(SaveError::Conflict)
        ));
        settings.save(1, view.config, None).unwrap();
        assert!(settings.authorized(secret));
        let restarted = AccessSettings::default();
        let active = restarted.configure(path, dir.defaults()).unwrap();
        assert_eq!(active.port, 9090);
        assert!(!restarted.view().unwrap().restart_required);
        assert!(restarted.authorized(secret));
        let new_secret = "a-second-management-key-replaces-the-first";
        restarted
            .save(2, restarted.view().unwrap().config, Some(new_secret.into()))
            .unwrap();
        assert!(!restarted.authorized(secret));
        assert!(restarted.authorized(new_secret));
        let mut config = restarted.view().unwrap().config;
        config.remote_management.allow_remote = false;
        restarted.save(3, config, Some(String::new())).unwrap();
        assert!(restarted.local_token().is_some());
        assert!(!restarted.authorized(new_secret));
    }

    #[test]
    fn invalid_and_conflicting_saves_preserve_disk_and_runtime() {
        let dir = Directory::new();
        let settings = AccessSettings::default();
        let path = dir.0.join("access.json");
        settings.configure(path.clone(), dir.defaults()).unwrap();
        settings
            .save(0, settings.view().unwrap().config, None)
            .unwrap();
        let original = std::fs::read(&path).unwrap();
        for mutate in [
            |c: &mut AccessConfig| c.port = 0,
            |c: &mut AccessConfig| {
                c.api_keys = vec!["duplicate-key".into(), "duplicate-key".into()]
            },
            |c: &mut AccessConfig| c.remote_management.allow_remote = true,
            |c: &mut AccessConfig| {
                c.remote_management.panel_github_repository = "https://localhost/owner/repo".into()
            },
            |c: &mut AccessConfig| c.tls.enable = true,
            |c: &mut AccessConfig| c.auth_dir = "relative/accounts".into(),
        ] {
            let mut config = settings.view().unwrap().config;
            mutate(&mut config);
            assert!(matches!(
                settings.save(1, config, None),
                Err(SaveError::Invalid(_))
            ));
            assert_eq!(std::fs::read(&path).unwrap(), original);
            assert_eq!(settings.view().unwrap().revision, 1);
        }
        std::fs::write(&path, b"externally edited").unwrap();
        assert!(matches!(
            settings.save(1, settings.view().unwrap().config, None),
            Err(SaveError::Conflict)
        ));
        assert_eq!(std::fs::read(&path).unwrap(), b"externally edited");
    }

    #[test]
    fn api_auth_accepts_provider_header_forms_and_strips_only_gateway_credentials() {
        let dir = Directory::new();
        let settings = AccessSettings::default();
        let mut config = dir.defaults();
        config.api_keys = vec!["gateway-test-key".into()];
        settings
            .configure(dir.0.join("access.json"), config)
            .unwrap();
        for (name, value) in [
            ("authorization", "bEaReR gateway-test-key"),
            ("x-api-key", "gateway-test-key"),
            ("x-goog-api-key", "gateway-test-key"),
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(name, value.parse().unwrap());
            headers.insert("x-upstream-auth", "preserved".parse().unwrap());
            let mut uri = "/v1/models?unrelated=a%20b".parse().unwrap();
            assert!(settings.authenticate_api(&mut headers, &mut uri));
            assert!(!headers.contains_key(name));
            assert_eq!(headers["x-upstream-auth"], "preserved");
            assert_eq!(uri.to_string(), "/v1/models?unrelated=a%20b");
        }
        let mut headers = HeaderMap::new();
        let mut uri = "/v1/models?key=gateway-test-key&keep=1".parse().unwrap();
        assert!(settings.authenticate_api(&mut headers, &mut uri));
        assert_eq!(uri.to_string(), "/v1/models?keep=1");
        headers.insert("authorization", "Bearer wrong-key".parse().unwrap());
        assert!(!settings.authenticate_api(&mut headers, &mut uri));
        assert_eq!(headers["authorization"], "Bearer wrong-key");
    }

    #[test]
    fn tls_validates_actual_certificate_key_pair_and_github_urls() {
        let dir = Directory::new();
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let other = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_path = dir.0.join("cert.pem");
        let key_path = dir.0.join("key.pem");
        std::fs::write(&cert_path, cert.serialize_pem().unwrap()).unwrap();
        std::fs::write(&key_path, cert.serialize_private_key_pem()).unwrap();
        let config = TlsSettings {
            enable: true,
            cert: cert_path.display().to_string(),
            key: key_path.display().to_string(),
        };
        assert!(tls_config(&config).unwrap().is_some());
        std::fs::write(&key_path, other.serialize_private_key_pem()).unwrap();
        assert!(tls_config(&config).is_err());
        assert_eq!(
            github_release_url("https://github.com/owner/repo").unwrap(),
            "https://api.github.com/repos/owner/repo/releases/latest"
        );
        assert!(
            github_release_url("https://api.github.com/repos/owner/repo/releases/latest").is_ok()
        );
        for value in [
            "http://github.com/owner/repo",
            "https://github.com/owner/repo?token=x",
            "https://github.com/../repo",
            "https://github.com/a/b/c",
        ] {
            assert!(github_release_url(value).is_err());
        }
    }
}
