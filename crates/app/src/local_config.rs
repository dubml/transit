use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::io::AsyncReadExt;
use transit_core::{RuntimeConfig, SourceId};
use transit_proxy::ProxyState;

const MAX_CONFIG_BYTES: u64 = 8 * 1024 * 1024;

pub struct LocalConfiguration {
    path: PathBuf,
    last_attempt: Vec<u8>,
}

impl LocalConfiguration {
    pub async fn load(path: PathBuf, state: &ProxyState) -> io::Result<Self> {
        let raw = read(&path).await?;
        publish(state, parse(&path, &raw)?)?;
        Ok(Self {
            path,
            last_attempt: raw,
        })
    }

    pub async fn watch(self, state: ProxyState, shutdown: impl std::future::Future<Output = ()>) {
        self.watch_interval(state, Duration::from_millis(500), shutdown)
            .await;
    }

    async fn watch_interval(
        mut self,
        state: ProxyState,
        period: Duration,
        shutdown: impl std::future::Future<Output = ()>,
    ) {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_error = String::new();
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                biased;
                _ = &mut shutdown => return,
                _ = interval.tick() => {}
            }
            let result = match read(&self.path).await {
                Ok(raw) if raw == self.last_attempt => continue,
                Ok(raw) => {
                    // Cache the attempted bytes, including malformed content,
                    // so unchanged invalid files are not parsed on every tick.
                    let config = parse(&self.path, &raw);
                    self.last_attempt = raw;
                    config.and_then(|config| publish(&state, config))
                }
                Err(error) => Err(error),
            };
            match result {
                Ok(()) => {
                    last_error.clear();
                    tracing::info!(path = %self.path.display(), "local configuration reloaded");
                }
                Err(error) => {
                    let message = error.to_string();
                    if message != last_error {
                        tracing::warn!(path = %self.path.display(), %error, "local configuration reload rejected; retaining last valid configuration");
                        last_error = message;
                    }
                }
            }
        }
    }
}

async fn read(path: &Path) -> io::Result<Vec<u8>> {
    let file = tokio::fs::File::open(path).await?;
    let mut raw = Vec::new();
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut raw)
        .await?;
    if raw.len() as u64 > MAX_CONFIG_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Local configuration exceeds 8 MiB",
        ));
    }
    Ok(raw)
}

fn parse(path: &Path, raw: &[u8]) -> io::Result<RuntimeConfig> {
    let config: RuntimeConfig = if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        serde_json::from_slice(raw).map_err(io::Error::other)?
    } else {
        serde_yaml::from_slice(raw).map_err(io::Error::other)?
    };
    config.validate().map_err(|errors| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid local configuration: {errors:?}"),
        )
    })?;
    Ok(config)
}

fn publish(state: &ProxyState, config: RuntimeConfig) -> io::Result<()> {
    if state
        .store()
        .snapshot()
        .source_versions()
        .contains_key(&SourceId::Xds)
    {
        return Err(io::Error::other(
            "Local configuration cannot override an active xDS source",
        ));
    }
    state
        .apply_config_from(SourceId::Static, config)
        .map_err(|errors| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Local configuration conflicts: {errors:?}"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Client, Response};
    use std::net::SocketAddr;
    use tokio::{sync::oneshot, task::JoinHandle};
    use transit_core::{
        AgentProtocol, AgentRoute, Backend, BackendKind, ConfigDelta, WeightedBackend,
    };
    use transit_proxy::ProxyServer;

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "transit-local-config-{}-{nonce}",
                std::process::id()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    struct Task(JoinHandle<()>);
    impl Drop for Task {
        fn drop(&mut self) {
            self.0.abort();
        }
    }

    fn upstream(body: &'static str) -> (SocketAddr, Task) {
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        let address = socket.local_addr().unwrap();
        let service = hyper::service::make_service_fn(move |_| async move {
            Ok::<_, std::convert::Infallible>(hyper::service::service_fn(move |_| async move {
                Ok::<_, std::convert::Infallible>(Response::new(Body::from(body)))
            }))
        });
        (
            address,
            Task(tokio::spawn(async move {
                hyper::Server::from_tcp(socket)
                    .unwrap()
                    .serve(service)
                    .await
                    .unwrap();
            })),
        )
    }

    fn config(address: SocketAddr, version: &str) -> RuntimeConfig {
        let mut config = RuntimeConfig::empty(version);
        config.backends.push(Backend {
            name: "upstream".into(),
            kind: BackendKind::Http {
                endpoint: format!("http://{address}"),
            },
            policies: vec![],
        });
        config.routes.push(AgentRoute {
            name: "route".into(),
            protocol: AgentProtocol::Http,
            listener_ports: vec![],
            matches: vec![],
            weighted_backends: vec![WeightedBackend {
                name: "upstream".into(),
                weight: 1,
                priority: None,
            }],
            policies: vec![],
            replace_prefix_match: None,
        });
        config
    }

    async fn wait_version(state: &ProxyState, version: &str) {
        tokio::time::timeout(Duration::from_secs(3), async {
            while state
                .snapshot()
                .source_versions()
                .get(&SourceId::Static)
                .map(String::as_str)
                != Some(version)
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }

    async fn response(address: SocketAddr) -> String {
        let client = Client::builder()
            .pool_max_idle_per_host(0)
            .build_http::<Body>();
        let response = client
            .get(format!("http://{address}/api").parse().unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        String::from_utf8(
            hyper::body::to_bytes(response.into_body())
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn file_updates_change_live_routes_and_reject_invalid_or_missing_input() {
        let directory = Directory::new();
        let path = directory.0.join("config.JSON");
        let (one, _first_upstream) = upstream("one");
        let (two, _second_upstream) = upstream("two");
        std::fs::write(&path, serde_json::to_vec(&config(one, "one")).unwrap()).unwrap();
        let state = ProxyState::new();
        let configuration = LocalConfiguration::load(path.clone(), &state)
            .await
            .unwrap();
        let (stop, shutdown) = oneshot::channel();
        let mut watcher = Task(tokio::spawn(configuration.watch_interval(
            state.clone(),
            Duration::from_millis(20),
            async {
                let _ = shutdown.await;
            },
        )));
        let address = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let proxy = ProxyServer::new(state.clone());
        let _proxy_task = Task(tokio::spawn(async move {
            proxy.serve(address).await.unwrap();
        }));
        tokio::time::timeout(Duration::from_secs(3), async {
            while tokio::net::TcpStream::connect(address).await.is_err() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(response(address).await, "one");
        let replacement = directory.0.join("replacement");
        std::fs::write(
            &replacement,
            serde_json::to_vec(&config(two, "two")).unwrap(),
        )
        .unwrap();
        std::fs::rename(replacement, &path).unwrap();
        wait_version(&state, "two").await;
        assert_eq!(response(address).await, "two");
        let revision = state.snapshot().revision();
        for invalid in [
            b"{invalid".to_vec(),
            serde_json::to_vec(&{
                let mut value = config(one, "bad-reference");
                value.backends.clear();
                value
            })
            .unwrap(),
        ] {
            std::fs::write(&path, invalid).unwrap();
            tokio::time::sleep(Duration::from_millis(80)).await;
            assert_eq!(state.snapshot().revision(), revision);
            assert_eq!(response(address).await, "two");
        }
        std::fs::remove_file(&path).unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(state.snapshot().revision(), revision);
        assert_eq!(response(address).await, "two");
        std::fs::write(&path, serde_json::to_vec(&config(one, "restored")).unwrap()).unwrap();
        wait_version(&state, "restored").await;
        assert_eq!(response(address).await, "one");
        let restarted = ProxyState::new();
        LocalConfiguration::load(path, &restarted).await.unwrap();
        assert_eq!(
            restarted.snapshot().to_runtime_config(),
            state.snapshot().to_runtime_config()
        );
        stop.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), &mut watcher.0)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn local_input_is_bounded_and_cannot_override_xds() {
        let directory = Directory::new();
        let path = directory.0.join("runtime.yaml");
        std::fs::write(&path, "version: yaml\nlisteners: []\n").unwrap();
        let state = ProxyState::new();
        state.apply_delta(
            SourceId::Xds,
            ConfigDelta::default().with_version("external"),
        );
        assert!(LocalConfiguration::load(path.clone(), &state)
            .await
            .is_err());
        assert!(!state
            .snapshot()
            .source_versions()
            .contains_key(&SourceId::Static));
        let local = ProxyState::new();
        LocalConfiguration::load(path.clone(), &local)
            .await
            .unwrap();
        assert_eq!(
            local.snapshot().source_versions()[&SourceId::Static],
            "yaml"
        );
        let file = std::fs::File::create(path.clone()).unwrap();
        file.set_len(MAX_CONFIG_BYTES + 1).unwrap();
        assert_eq!(
            read(&path).await.unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            local.snapshot().source_versions()[&SourceId::Static],
            "yaml"
        );
    }
}
