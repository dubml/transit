use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use transit_core::prelude::*;

pub mod app;
pub mod config;
pub mod management;
pub mod serdes;
pub mod state_manager;

#[derive(Clone, Debug)]
pub enum ConfigSource {
    File(PathBuf),
    Static(Bytes),
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Address {
    Off,
    Localhost(bool, u16),
    SocketAddr(SocketAddr),
}

impl Address {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        if value == "off" {
            return Ok(Self::Off);
        }
        if let Some(port) = value.strip_prefix("localhost:") {
            return Ok(Self::Localhost(true, port.parse()?));
        }
        Ok(Self::SocketAddr(value.parse()?))
    }

    pub fn socket_addrs(&self) -> Vec<SocketAddr> {
        match self {
            Self::Off => Vec::new(),
            Self::Localhost(ipv6, port) => {
                let mut addrs = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), *port)];
                if *ipv6 {
                    addrs.push(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), *port));
                }
                addrs
            }
            Self::SocketAddr(addr) => vec![*addr],
        }
    }
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
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub admin_addr: Address,
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub health_addr: Address,
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
            admin_addr: Address::Localhost(true, 26000),
            health_addr: Address::SocketAddr(SocketAddr::from(([0, 0, 0, 0], 26021))),
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
