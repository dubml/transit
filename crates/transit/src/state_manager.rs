use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::error::{Result, TransitError};
use crate::proxy::ProxyServer;
use crate::RuntimeConfig;

pub const DEFAULT_CONFIG_YAML: &str = r#"version: v1
routes:
  - name: default
    port: 6010
    matches:
      - pathPrefix: /
    backends:
      - 127.0.0.1:8080
"#;

pub fn is_kubernetes_environment() -> bool {
    std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
        || Path::new("/var/run/secrets/kubernetes.io").exists()
}

#[derive(Clone, Debug)]
pub enum ConfigSource {
    File(PathBuf),
    Stdin,
    EmptyDefault,
}

pub struct StateManager {
    source: ConfigSource,
    current_config: Arc<std::sync::RwLock<Arc<RuntimeConfig>>>,
}

impl StateManager {
    pub fn init(specified_path: Option<&Path>) -> Result<(Self, String)> {
        let (manager, desc) = match specified_path {
            Some(path) if path == Path::new("-") => {
                let mut buffer = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)
                    .map_err(TransitError::Io)?;
                let cfg = parse_config_str(&buffer, "stdin")?;
                let desc = "standard input".to_string();
                let manager = Self {
                    source: ConfigSource::Stdin,
                    current_config: Arc::new(std::sync::RwLock::new(Arc::new(cfg))),
                };
                (manager, desc)
            }
            Some(path) => {
                let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let cfg = load_config_file(&resolved)?;
                let desc = resolved.display().to_string();
                let manager = Self {
                    source: ConfigSource::File(resolved),
                    current_config: Arc::new(std::sync::RwLock::new(Arc::new(cfg))),
                };
                (manager, desc)
            }
            None => {
                let default_path = PathBuf::from("config.yaml");
                if default_path.exists() {
                    let resolved = std::fs::canonicalize(&default_path).unwrap_or_else(|_| default_path);
                    let cfg = load_config_file(&resolved)?;
                    let desc = resolved.display().to_string();
                    let manager = Self {
                        source: ConfigSource::File(resolved),
                        current_config: Arc::new(std::sync::RwLock::new(Arc::new(cfg))),
                    };
                    (manager, desc)
                } else if is_kubernetes_environment() {
                    warn!(
                        path = ?default_path,
                        "Configuration file not found in Kubernetes environment, starting with empty configuration"
                    );
                    let cfg = RuntimeConfig::empty("default");
                    let desc = "empty default".to_string();
                    let manager = Self {
                        source: ConfigSource::EmptyDefault,
                        current_config: Arc::new(std::sync::RwLock::new(Arc::new(cfg))),
                    };
                    (manager, desc)
                } else {
                    if let Err(err) = std::fs::write(&default_path, DEFAULT_CONFIG_YAML) {
                        warn!(
                            path = ?default_path,
                            %err,
                            "Failed to auto-create default config.yaml, falling back to empty in-memory configuration"
                        );
                        let cfg = RuntimeConfig::empty("default");
                        let desc = "empty default".to_string();
                        let manager = Self {
                            source: ConfigSource::EmptyDefault,
                            current_config: Arc::new(std::sync::RwLock::new(Arc::new(cfg))),
                        };
                        info!(target: "state_manager", "loaded config from {:?}", manager.source);
                        return Ok((manager, desc));
                    }
                    info!(path = %default_path.display(), "Auto-created default configuration file");
                    let resolved = std::fs::canonicalize(&default_path).unwrap_or_else(|_| default_path);
                    let cfg = load_config_file(&resolved)?;
                    let desc = resolved.display().to_string();
                    let manager = Self {
                        source: ConfigSource::File(resolved),
                        current_config: Arc::new(std::sync::RwLock::new(Arc::new(cfg))),
                    };
                    (manager, desc)
                }
            }
        };

        info!(target: "state_manager", "loaded config from {:?}", manager.source);
        Ok((manager, desc))
    }

    pub fn current_config(&self) -> Arc<RuntimeConfig> {
        self.current_config.read().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn update_config(&self, new_config: Arc<RuntimeConfig>) {
        let mut w = self.current_config.write().unwrap();
        *w = new_config;
    }

    pub fn start_watcher(&self, server: ProxyServer) -> Option<JoinHandle<()>> {
        let path = match &self.source {
            ConfigSource::File(p) => p.clone(),
            _ => return None,
        };

        info!(target: "state_manager", "Watching config file: {}", path.display());

        let mut last_hash = match std::fs::read_to_string(&path) {
            Ok(content) => calculate_hash(&content),
            Err(_) => 0,
        };

        let current_lock = self.current_config.clone();

        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(200));
            loop {
                interval.tick().await;

                let content = match tokio::fs::read_to_string(&path).await {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let current_hash = calculate_hash(&content);
                if current_hash == last_hash {
                    continue;
                }

                match parse_config_str(&content, &path.display().to_string()) {
                    Ok(new_cfg) => {
                        if let Err(conflicts) = new_cfg.validate() {
                            warn!(
                                path = %path.display(),
                                ?conflicts,
                                "Configuration reload ignored due to validation conflicts"
                            );
                            last_hash = current_hash;
                            continue;
                        }

                        let arc_cfg = Arc::new(new_cfg);
                        {
                            let mut w = current_lock.write().unwrap();
                            *w = arc_cfg.clone();
                        }
                        server.update_config(arc_cfg);
                        last_hash = current_hash;
                        info!(path = %path.display(), "Configuration reloaded successfully");
                    }
                    Err(err) => {
                        warn!(
                            path = %path.display(),
                            %err,
                            "Configuration reload failed to parse file, keeping previous configuration"
                        );
                        last_hash = current_hash;
                    }
                }
            }
        }))
    }
}

pub fn load_config_file(path: &Path) -> Result<RuntimeConfig> {
    let content = std::fs::read_to_string(path).map_err(|err| {
        TransitError::InvalidConfig(format!(
            "Failed to read configuration file {}: {err}",
            path.display()
        ))
    })?;
    parse_config_str(&content, &path.display().to_string())
}

pub fn parse_config_str(content: &str, source_name: &str) -> Result<RuntimeConfig> {
    if let Ok(cfg) = serde_json::from_str::<RuntimeConfig>(content) {
        return Ok(cfg);
    }
    serde_yaml::from_str::<RuntimeConfig>(content).map_err(|err| {
        TransitError::InvalidConfig(format!(
            "Failed to parse configuration from {source_name} as YAML or JSON: {err}"
        ))
    })
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
