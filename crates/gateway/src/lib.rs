use std::path::PathBuf;
use std::time::Duration;
use transit_core::prelude::*;

pub mod app;
pub mod config;
pub mod serdes;

#[derive(Clone, Debug)]
pub enum ConfigSource {
    File(PathBuf),
    Static(Bytes),
}

impl serde::Serialize for ConfigSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::File(path) => serializer.serialize_str(&path.to_string_lossy()),
            Self::Static(_) => serializer.serialize_str("static"),
        }
    }
}

#[derive(serde::Serialize, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    #[serde(with = "duration_serde")]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub termination_max_deadline: Duration,
    #[serde(with = "duration_serde")]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub termination_min_deadline: Duration,
    pub num_worker_threads: usize,
    pub xds: XdsConfig,
    pub storage: StorageConfig,
}

#[derive(serde::Serialize, Clone, Debug, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct XdsConfig {
    #[cfg_attr(feature = "schema", schemars(with = "Option<String>"))]
    pub local_config: Option<ConfigSource>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct StorageConfig {
    pub mode: ConfigStoreMode,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ConfigStoreMode {
    #[default]
    File,
    Database,
    ReadOnly,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            termination_max_deadline: Duration::from_secs(5),
            termination_min_deadline: Duration::ZERO,
            num_worker_threads: 3,
            xds: XdsConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

pub(crate) mod duration_serde {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}s", value.as_secs()))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let (number, unit) = value.trim().trim_end_matches(char::is_whitespace).split_at(
            value
                .trim()
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(value.trim().len()),
        );
        let amount: u64 = number.parse().map_err(de::Error::custom)?;
        match unit.trim() {
            "s" => Ok(Duration::from_secs(amount)),
            "ms" => Ok(Duration::from_millis(amount)),
            "m" => amount
                .checked_mul(60)
                .map(Duration::from_secs)
                .ok_or_else(|| de::Error::custom("duration overflow")),
            "h" => amount
                .checked_mul(3600)
                .map(Duration::from_secs)
                .ok_or_else(|| de::Error::custom("duration overflow")),
            _ => Err(de::Error::custom("duration must use s, ms, m, or h")),
        }
    }
}
