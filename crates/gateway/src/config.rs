use std::path::PathBuf;

use serde::Deserialize;

use crate::{Config, ConfigSource, XdsConfig};

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawConfig {
    #[serde(
        default = "default_termination_max_deadline",
        with = "crate::duration_serde"
    )]
    termination_max_deadline: std::time::Duration,
    #[serde(default, with = "crate::duration_serde")]
    termination_min_deadline: std::time::Duration,
    #[serde(default = "default_num_worker_threads")]
    num_worker_threads: usize,
    admin_addr: Option<String>,
    health_addr: Option<String>,
    xds: RawXdsConfig,
    storage: crate::StorageConfig,
}

fn default_termination_max_deadline() -> std::time::Duration {
    std::time::Duration::from_secs(5)
}

fn default_num_worker_threads() -> usize {
    3
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RawXdsConfig {
    local_config: Option<PathBuf>,
}

pub fn parse_config(
    contents: String,
    local_config_source: Option<ConfigSource>,
) -> anyhow::Result<Config> {
    if contents.trim().is_empty() {
        return Ok(Config::default());
    }
    let raw: RawConfig = crate::serdes::yamlviajson::from_str(&contents)?;
    if raw.num_worker_threads == 0 {
        anyhow::bail!("numWorkerThreads must be greater than zero");
    }
    let local_config = raw
        .xds
        .local_config
        .map(ConfigSource::File)
        .or(local_config_source);
    Ok(Config {
        termination_max_deadline: raw.termination_max_deadline,
        termination_min_deadline: raw.termination_min_deadline,
        num_worker_threads: raw.num_worker_threads,
        admin_addr: raw
            .admin_addr
            .as_deref()
            .map(crate::Address::parse)
            .transpose()?
            .unwrap_or_else(|| Config::default().admin_addr),
        health_addr: raw
            .health_addr
            .as_deref()
            .map(crate::Address::parse)
            .transpose()?
            .unwrap_or_else(|| Config::default().health_addr),
        xds: XdsConfig { local_config },
        storage: raw.storage,
    })
}
