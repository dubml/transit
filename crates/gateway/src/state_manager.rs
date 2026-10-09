use std::{sync::Arc, time::Duration};

use anyhow::Context;
use tokio::sync::watch;
use tracing::{info, warn};

use crate::ConfigSource;

#[derive(Clone, Debug)]
pub struct ConfigSnapshot {
    pub revision: u64,
    pub contents: String,
    pub document: serde_yaml::Value,
}

pub struct StateManager {
    source: ConfigSource,
    current: watch::Sender<Arc<ConfigSnapshot>>,
}

impl StateManager {
    pub async fn new(source: ConfigSource) -> anyhow::Result<Self> {
        let contents = read_source(&source).await?;
        let document =
            crate::serdes::yamlviajson::from_str(&contents).context("parse local configuration")?;
        info!(source = ?source, "Loaded configuration");
        let initial = Arc::new(ConfigSnapshot {
            revision: 1,
            contents,
            document,
        });
        let (current, _) = watch::channel(initial);
        Ok(Self { source, current })
    }

    pub fn subscribe(&self) -> watch::Receiver<Arc<ConfigSnapshot>> {
        self.current.subscribe()
    }

    pub async fn run(self) {
        let ConfigSource::File(path) = &self.source else {
            return;
        };
        info!(path = %path.display(), "Watching config file");
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let result = async {
                let contents = read_source(&self.source).await?;
                let document = crate::serdes::yamlviajson::from_str(&contents)
                    .context("parse local configuration")?;
                anyhow::Ok((contents, document))
            }
            .await;

            match result {
                Ok((contents, document)) => {
                    let previous = self.current.borrow().clone();
                    if contents != previous.contents {
                        self.current.send_replace(Arc::new(ConfigSnapshot {
                            revision: previous.revision + 1,
                            contents,
                            document,
                        }));
                    }
                }
                Err(error) => {
                    warn!(%error, "Configuration reload failed; keeping the last valid configuration")
                }
            }
        }
    }
}

async fn read_source(source: &ConfigSource) -> anyhow::Result<String> {
    match source {
        ConfigSource::File(path) => tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("read local configuration {}", path.display())),
        ConfigSource::Static(bytes) => {
            String::from_utf8(bytes.to_vec()).context("local configuration is not valid UTF-8")
        }
    }
}
