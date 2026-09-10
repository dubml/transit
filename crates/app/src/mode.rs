#![allow(dead_code)]

use crate::Args;
use transit_core::{ConfigurationSource, RuntimeMode, RuntimeModeInfo, RuntimeRole};

#[derive(Debug, PartialEq, Eq)]
pub struct StartupPlan {
    pub runtime: RuntimeModeInfo,
    pub xds_endpoint: Option<String>,
}

impl StartupPlan {
    pub fn from_args(args: &Args) -> Result<Self, String> {
        let endpoint = args.xds_address.trim();
        let uses_xds = args
            .xds_enabled
            .unwrap_or(args.mode == RuntimeMode::Kubernetes || !endpoint.is_empty());
        if !uses_xds && !endpoint.is_empty() {
            return Err("--xds-address conflicts with --xds-enabled=false".into());
        }
        if args.mode == RuntimeMode::Kubernetes && (!uses_xds || args.static_config.is_some()) {
            return Err("the Kubernetes data plane receives xDS; local static configuration is not allowed".into());
        }
        if uses_xds && args.static_config.is_some() {
            return Err("choose one configuration source: --static-config or xDS".into());
        }
        let endpoint = if uses_xds {
            if endpoint.is_empty() {
                if args.mode == RuntimeMode::Standalone {
                    return Err("--xds-enabled=true requires an explicit --xds-address in standalone mode".into());
                }
                Some("http://transit-control-plane.transit-system.svc:18000".into())
            } else {
                if !(endpoint.starts_with("http://") || endpoint.starts_with("https://")) {
                    return Err("--xds-address must start with http:// or https://".into());
                }
                Some(endpoint.to_owned())
            }
        } else {
            None
        };
        Ok(Self {
            runtime: RuntimeModeInfo {
                mode: args.mode,
                role: RuntimeRole::DataPlane,
                source: if endpoint.is_some() {
                    ConfigurationSource::Xds
                } else {
                    ConfigurationSource::Local
                },
            },
            xds_endpoint: endpoint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn args(values: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once("transit").chain(values.iter().copied())).unwrap()
    }

    #[test]
    fn standalone_does_not_implicitly_connect_to_a_cluster() {
        let plan = StartupPlan::from_args(&args(&[])).unwrap();
        assert_eq!(plan.runtime, RuntimeModeInfo::default());
        assert_eq!(plan.xds_endpoint, None);
        let plan = StartupPlan::from_args(&args(&["--static-config", "local.yaml"])).unwrap();
        assert_eq!(plan.runtime.source, ConfigurationSource::Local);
    }

    #[test]
    fn kubernetes_data_plane_uses_the_independent_controller() {
        let data = StartupPlan::from_args(&args(&["--mode", "kubernetes"])).unwrap();
        assert_eq!(data.runtime.source, ConfigurationSource::Xds);
        assert!(data.xds_endpoint.unwrap().contains("transit-control-plane"));
    }

    #[test]
    fn dubbod_delegation_is_an_explicit_source_in_either_environment() {
        for mode in ["standalone", "kubernetes"] {
            let endpoint = "https://dubbod.dubbo-system.svc:26012";
            let plan = StartupPlan::from_args(&args(&[
                "--mode", mode, "--xds-address", endpoint,
            ])).unwrap();
            assert_eq!(plan.xds_endpoint.as_deref(), Some(endpoint));
            assert_eq!(plan.runtime.source, ConfigurationSource::Xds);
        }
    }

    #[test]
    fn conflicting_sources_fail_before_starting_services() {
        for options in [
            vec!["--mode", "kubernetes", "--static-config", "local.yaml"],
            vec!["--mode", "kubernetes", "--xds-enabled=false"],
            vec!["--xds-enabled=true"],
            vec!["--static-config", "local.yaml", "--xds-address", "http://dubbod:26012"],
            vec!["--xds-address", "http://dubbod:26012", "--xds-enabled=false"],
        ] {
            assert!(StartupPlan::from_args(&args(&options)).is_err(), "{options:?}");
        }
    }
}
