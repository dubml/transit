use crate::{Result, TransitError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
pub struct MatchInput<'a> {
    pub host: &'a str,
    pub path: &'a str,
    pub headers: &'a [(String, String)],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigConflict {
    pub kind: String,
    pub message: String,
}

impl ConfigConflict {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}

pub const HTTP_LISTENER_PORT: u16 = 6010;
pub const HTTPS_LISTENER_PORT: u16 = 26443;
pub const UI_PORT: u16 = 26021;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub ports: Vec<PortConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PortConfig {
    #[serde(default = "default_port_number")]
    pub default_port: u16,
    #[serde(default)]
    pub listeners: Vec<ListenerConfig>,
}

fn default_port_number() -> u16 {
    HTTP_LISTENER_PORT
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListenerConfig {
    pub name: String,
    #[serde(default)]
    pub protocol: ListenerProtocol,
    #[serde(default)]
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ListenerProtocol {
    #[default]
    Http,
    Https,
    Tcp,
    Tls,
    Grpc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteConfig {
    pub name: String,
    #[serde(default)]
    pub matches: Vec<RouteMatch>,
    #[serde(default)]
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub policies: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteMatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathMatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HeaderMatch>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PathMatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeaderMatch {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub present: Option<bool>,
}

impl RouteMatch {
    pub fn matches(&self, input: &MatchInput<'_>) -> bool {
        if let Some(path_match) = &self.path {
            if let Some(prefix) = &path_match.prefix {
                if !input.path.starts_with(prefix) {
                    return false;
                }
            }
            if let Some(exact) = &path_match.exact {
                if input.path != exact {
                    return false;
                }
            }
        }

        if let Some(methods) = &self.methods {
            if !methods.iter().any(|m| m.eq_ignore_ascii_case(input.host)) {
                // If methods are specified, input method matching would be checked if passed
            }
        }

        if let Some(header_matches) = &self.headers {
            for hm in header_matches {
                let val = input
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&hm.name))
                    .map(|(_, v)| v.as_str());

                if let Some(present) = hm.present {
                    if present != val.is_some() {
                        return false;
                    }
                }
                if let Some(exact) = &hm.exact {
                    if val != Some(exact.as_str()) {
                        return false;
                    }
                }
                if let Some(prefix) = &hm.prefix {
                    match val {
                        Some(v) if v.starts_with(prefix) => {}
                        _ => return false,
                    }
                }
            }
        }

        true
    }
}

impl RouteConfig {
    pub fn matches(&self, input: &MatchInput<'_>) -> bool {
        if self.matches.is_empty() {
            return true;
        }
        self.matches.iter().any(|m| m.matches(input))
    }
}

impl RuntimeConfig {
    pub fn empty(version: impl Into<String>) -> Self {
        Self {
            version: Some(version.into()),
            ports: Vec::new(),
        }
    }

    pub fn validate(&self) -> std::result::Result<(), Vec<ConfigConflict>> {
        let mut conflicts = Vec::new();
        let mut ports_seen = BTreeSet::new();

        for port in &self.ports {
            if !ports_seen.insert(port.default_port) {
                conflicts.push(ConfigConflict::new(
                    "duplicate-port",
                    format!("port {} is defined more than once", port.default_port),
                ));
            }

            let mut listener_names = BTreeSet::new();
            for listener in &port.listeners {
                if !listener_names.insert(&listener.name) {
                    conflicts.push(ConfigConflict::new(
                        "duplicate-listener",
                        format!(
                            "listener {} is defined more than once on port {}",
                            listener.name, port.default_port
                        ),
                    ));
                }
            }
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(conflicts)
        }
    }

    pub fn route_for(&self, port: u16, input: &MatchInput<'_>) -> Result<&RouteConfig> {
        // First match on exact port if specified
        for p in &self.ports {
            if p.default_port == port || port == 0 {
                for listener in &p.listeners {
                    for route in &listener.routes {
                        if route.matches(input) {
                            return Ok(route);
                        }
                    }
                }
            }
        }

        // Fallback: match across all ports
        for p in &self.ports {
            for listener in &p.listeners {
                for route in &listener.routes {
                    if route.matches(input) {
                        return Ok(route);
                    }
                }
            }
        }

        Err(TransitError::RouteNotFound {
            host: input.host.to_string(),
            path: input.path.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_yaml_config() {
        let yaml = r#"
ports:
- defaultPort: 6010
  listeners:
  - name: http-listener
    protocol: http
    routes:
    - name: api-route
      matches:
      - path: { prefix: /api }
      endpoints:
      - 127.0.0.1:8080
      policies: []
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.ports.len(), 1);
        let port = &config.ports[0];
        assert_eq!(port.default_port, 6010);
        assert_eq!(port.listeners.len(), 1);

        let listener = &port.listeners[0];
        assert_eq!(listener.name, "http-listener");
        assert_eq!(listener.protocol, ListenerProtocol::Http);
        assert_eq!(listener.routes.len(), 1);

        let route = &listener.routes[0];
        assert_eq!(route.name, "api-route");
        assert_eq!(route.endpoints, vec!["127.0.0.1:8080"]);
        assert_eq!(route.policies.len(), 0);

        let input_matched = MatchInput {
            host: "localhost",
            path: "/api/v1/users",
            headers: &[],
        };
        assert!(route.matches(&input_matched));

        let route_found = config.route_for(6010, &input_matched).expect("route_for");
        assert_eq!(route_found.name, "api-route");

        let input_unmatched = MatchInput {
            host: "localhost",
            path: "/healthz",
            headers: &[],
        };
        assert!(!route.matches(&input_unmatched));
    }
}
