use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Deployment environment. Configuration ownership is represented separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeMode {
    #[default]
    Standalone,
    Kubernetes,
}

impl FromStr for RuntimeMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "standalone" => Ok(Self::Standalone),
            "kubernetes" => Ok(Self::Kubernetes),
            _ => Err("mode must be standalone or kubernetes".into()),
        }
    }
}

impl fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Standalone => "standalone",
            Self::Kubernetes => "kubernetes",
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeRole {
    #[default]
    DataPlane,
    ControlPlane,
}

impl FromStr for RuntimeRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "data-plane" => Ok(Self::DataPlane),
            "control-plane" => Ok(Self::ControlPlane),
            _ => Err("role must be data-plane or control-plane".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationSource {
    #[default]
    Local,
    Kubernetes,
    Xds,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeModeInfo {
    pub mode: RuntimeMode,
    pub role: RuntimeRole,
    pub source: ConfigurationSource,
}

impl RuntimeModeInfo {
    pub fn local_configuration_writable(&self) -> bool {
        self.mode == RuntimeMode::Standalone && self.source == ConfigurationSource::Local
    }
}
