use std::path::PathBuf;
use transit_core::prelude::*;

pub mod config;
pub mod serdes;

#[derive(Clone, Debug)]
pub enum ConfigSource {
    File(PathBuf),
    Static(Bytes),
}

#[derive(serde::Serialize, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Config {}
