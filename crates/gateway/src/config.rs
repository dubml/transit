use crate::{Config, ConfigSource};

pub fn parse_config(
    _contents: String,
    _local_config_source: Option<ConfigSource>,
) -> anyhow::Result<Config> {
    Ok(Config {})
}