use crate::{Result, TransitError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

pub const LLM_DEFAULT_PORT: u16 = 6020;
pub const MCP_DEFAULT_PORT: u16 = 6030;
pub const HTTP_LISTENER_PORT: u16 = 6010;
pub const HTTPS_LISTENER_PORT: u16 = 26443;
pub const UI_PORT: u16 = 26021;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpConfig>,
    #[serde(
        default,
        deserialize_with = "deserialize_one_or_many_ports",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub ports: Vec<PortConfig>,
}

fn deserialize_one_or_many_ports<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<PortConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(PortConfig),
        Many(Vec<PortConfig>),
    }

    match Option::<OneOrMany>::deserialize(deserializer)? {
        Some(OneOrMany::One(one)) => Ok(vec![one]),
        Some(OneOrMany::Many(many)) => Ok(many),
        None => Ok(Vec::new()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    #[serde(default = "default_llm_port")]
    pub default_port: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<LlmProviderConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<LlmModelItem>,
}

fn default_llm_port() -> u16 {
    LLM_DEFAULT_PORT
}

impl LlmConfig {
    pub fn subscriptions(&self) -> Vec<&LlmModelSubscription> {
        self.models
            .iter()
            .filter_map(|item| match item {
                LlmModelItem::Subscription(sub) => Some(sub),
                _ => None,
            })
            .collect()
    }

    pub fn providers(&self) -> Vec<&LlmProviderConfig> {
        let mut list: Vec<&LlmProviderConfig> = self.providers.iter().collect();
        for item in &self.models {
            if let LlmModelItem::Provider(p) = item {
                list.push(p);
            }
        }
        list
    }

    pub fn find_model(&self, model_name: &str) -> Option<(&LlmProviderConfig, &LlmProviderModel)> {
        for provider in self.providers() {
            for model in &provider.models {
                if model.name.eq_ignore_ascii_case(model_name)
                    || model
                        .model
                        .as_deref()
                        .is_some_and(|m| m.eq_ignore_ascii_case(model_name))
                {
                    return Some((provider, model));
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum LlmModelItem {
    Subscription(LlmModelSubscription),
    Provider(LlmProviderConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderConfig {
    pub provider: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<LlmProviderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderModel {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl LlmProviderModel {
    pub fn resolved_api_key(&self) -> Option<String> {
        self.api_key.as_ref().map(|key| {
            if let Some(var_name) = key.strip_prefix('$') {
                std::env::var(var_name).unwrap_or_else(|_| key.clone())
            } else {
                key.clone()
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LlmModelSubscription {
    pub subscribe: String,
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_ignore: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_alias: Vec<ModelAlias>,
}

impl LlmModelSubscription {
    pub fn is_model_ignored(&self, model: &str) -> bool {
        self.model_ignore
            .iter()
            .any(|pattern| pattern.eq_ignore_ascii_case(model))
    }

    pub fn resolve_alias<'a>(&'a self, requested: &'a str) -> Option<&'a str> {
        self.model_alias
            .iter()
            .find(|alias| alias.src.eq_ignore_ascii_case(requested))
            .map(|alias| alias.cur.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelAlias {
    pub src: String,
    pub cur: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    #[serde(default = "default_mcp_port")]
    pub default_port: u16,
    #[serde(default)]
    pub session_mode: McpSessionMode,
    #[serde(default)]
    pub prefix_policy: McpPrefixPolicy,
    #[serde(default)]
    pub failure_policy: McpFailurePolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<serde_json::Value>,
    #[serde(default)]
    pub targets: Vec<McpTarget>,
}

fn default_mcp_port() -> u16 {
    MCP_DEFAULT_PORT
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum McpSessionMode {
    #[default]
    Stateful,
    Stateless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum McpPrefixPolicy {
    #[default]
    Conditional,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum McpFailurePolicy {
    #[default]
    FailClosed,
    FailOpen,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpTarget {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdio: Option<McpStdioTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpHttpTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openapi: Option<McpOpenApiTarget>,
}

impl McpTarget {
    pub fn target_kind_str(&self) -> &'static str {
        if self.stdio.is_some() {
            "stdio"
        } else if self.mcp.is_some() {
            "mcp"
        } else if self.openapi.is_some() {
            "openapi"
        } else {
            "unknown"
        }
    }

    pub fn format_tool_name(
        &self,
        prefix_policy: McpPrefixPolicy,
        total_targets: usize,
        raw_name: &str,
    ) -> String {
        match prefix_policy {
            McpPrefixPolicy::Always => format!("{}__{}", self.name, raw_name),
            McpPrefixPolicy::Conditional => {
                if total_targets > 1 {
                    format!("{}__{}", self.name, raw_name)
                } else {
                    raw_name.to_string()
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpStdioTarget {
    pub cmd: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub clear_env: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpHttpTarget {
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpOpenApiTarget {
    pub host: String,
    pub schema: McpOpenApiSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpOpenApiSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListenerConfig {
    #[serde(default = "default_listener_name")]
    pub name: String,
    #[serde(default)]
    pub protocol: ListenerProtocol,
    #[serde(default)]
    pub routes: Vec<RouteConfig>,
}

fn default_listener_name() -> String {
    "default".to_string()
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteConfig {
    #[serde(default = "default_route_name")]
    pub name: String,
    #[serde(default)]
    pub matches: Vec<RouteMatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a2a: Option<A2aConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endpoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backends: Vec<RouteBackend>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<serde_json::Value>,
}

fn default_route_name() -> String {
    "default-route".to_string()
}

impl RouteConfig {
    pub fn target_hosts(&self) -> Vec<&str> {
        let mut hosts = Vec::new();
        for ep in &self.endpoints {
            hosts.push(ep.as_str());
        }
        for b in &self.backends {
            hosts.push(b.host_str());
        }
        hosts
    }

    pub fn is_a2a_enabled(&self) -> bool {
        self.a2a.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct A2aConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum RouteBackend {
    Host { host: String },
    Address(String),
}

impl RouteBackend {
    pub fn host_str(&self) -> &str {
        match self {
            RouteBackend::Host { host } => host.as_str(),
            RouteBackend::Address(addr) => addr.as_str(),
        }
    }
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
    #[serde(default, alias = "pathPrefix", skip_serializing_if = "Option::is_none")]
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
            llm: None,
            mcp: None,
            ports: Vec::new(),
        }
    }

    pub fn validate(&self) -> std::result::Result<(), Vec<ConfigConflict>> {
        let mut conflicts = Vec::new();
        let mut ports_seen = BTreeSet::new();

        if let Some(llm) = &self.llm {
            ports_seen.insert(llm.default_port);
            let mut ids_seen = BTreeSet::new();
            for sub in llm.subscriptions() {
                if !ids_seen.insert(sub.id.as_str()) {
                    conflicts.push(ConfigConflict::new(
                        "duplicate-llm-model-id",
                        format!("llm model subscription id {} is duplicated", sub.id),
                    ));
                }
            }

            let mut provider_names_seen = BTreeSet::new();
            for provider in llm.providers() {
                if !provider_names_seen.insert(provider.name.as_str()) {
                    conflicts.push(ConfigConflict::new(
                        "duplicate-llm-provider-name",
                        format!("llm provider name {} is duplicated", provider.name),
                    ));
                }
            }
        }

        if let Some(mcp) = &self.mcp {
            if !ports_seen.insert(mcp.default_port) {
                conflicts.push(ConfigConflict::new(
                    "port-conflict",
                    format!(
                        "mcp defaultPort {} conflicts with another configured port",
                        mcp.default_port
                    ),
                ));
            }

            let mut target_names_seen = BTreeSet::new();
            for target in &mcp.targets {
                if !target_names_seen.insert(target.name.as_str()) {
                    conflicts.push(ConfigConflict::new(
                        "duplicate-mcp-target-name",
                        format!("mcp target name {} is duplicated", target.name),
                    ));
                }

                let configured_count = [
                    target.stdio.is_some(),
                    target.mcp.is_some(),
                    target.openapi.is_some(),
                ]
                .iter()
                .filter(|&&c| c)
                .count();

                if configured_count == 0 {
                    conflicts.push(ConfigConflict::new(
                        "invalid-mcp-target",
                        format!(
                            "mcp target {} must specify one of stdio, mcp, or openapi",
                            target.name
                        ),
                    ));
                } else if configured_count > 1 {
                    conflicts.push(ConfigConflict::new(
                        "invalid-mcp-target",
                        format!(
                            "mcp target {} must specify only one of stdio, mcp, or openapi",
                            target.name
                        ),
                    ));
                }
            }
        }

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
    fn test_deserialize_llm_provider_and_models() {
        let yaml = r#"
llm:
  defaultPort: 6020
  models:
  - provider: openAI
    name: gpt-4o
    models:
    - name: gpt-4o
      apiKey: $OPENAI_API_KEY
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        let llm = config.llm.as_ref().expect("llm section exists");
        assert_eq!(llm.default_port, 6020);
        assert_eq!(llm.providers().len(), 1);

        let provider = llm.providers()[0];
        assert_eq!(provider.provider, "openAI");
        assert_eq!(provider.name, "gpt-4o");
        assert_eq!(provider.models.len(), 1);

        let m = &provider.models[0];
        assert_eq!(m.name, "gpt-4o");
        assert_eq!(m.api_key, Some("$OPENAI_API_KEY".to_string()));

        let (p, found_m) = llm.find_model("gpt-4o").expect("found model");
        assert_eq!(p.name, "gpt-4o");
        assert_eq!(found_m.name, "gpt-4o");
    }

    #[test]
    fn test_deserialize_a2a_route_config() {
        let yaml = r#"
ports:
  defaultPort: 6010
  listeners:
  - routes:
    - name: a2a-agent-route
      matches:
      - path:
          prefix: /a2a
      # 标记该路由启用 A2A 协议处理引擎
      a2a: {}
      policies: []
      backends:
      - host: localhost:9999
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.ports.len(), 1);
        let port = &config.ports[0];
        assert_eq!(port.default_port, 6010);
        assert_eq!(port.listeners.len(), 1);

        let listener = &port.listeners[0];
        assert_eq!(listener.name, "default");
        assert_eq!(listener.protocol, ListenerProtocol::Http);
        assert_eq!(listener.routes.len(), 1);

        let route = &listener.routes[0];
        assert_eq!(route.name, "a2a-agent-route");
        assert!(route.is_a2a_enabled());
        assert_eq!(route.backends.len(), 1);
        assert_eq!(route.target_hosts(), vec!["localhost:9999"]);

        let input_a2a = MatchInput {
            host: "localhost",
            path: "/a2a/v1/tasks",
            headers: &[],
        };
        assert!(route.matches(&input_a2a));

        let found = config.route_for(6010, &input_a2a).expect("found a2a route");
        assert_eq!(found.name, "a2a-agent-route");
    }

    #[test]
    fn test_deserialize_mcp_config() {
        let yaml = r#"
mcp:
  defaultPort: 6030
  sessionMode: stateful # 会话状态管理模式 stateful (默认) | stateless
  prefixPolicy: always  # 工具名前缀模式: conditional (默认, 多个 target 时添加 target_name__ 前缀) | always
  failurePolicy: failOpen  # 容错模式: failClosed (默认) | failOpen
  policies: []
  targets:
  # 方式 1: stdio 本地子进程 (由 Gateway 负责进程拉起与 stdin/stdout 通信)
  - name: time-tool
    stdio:
      cmd: uvx
      args:
      - mcp-server-time
      env:
        TZ: UTC
      clearEnv: false
  # 方式 2: mcp 原生 Streamable HTTP 服务
  - name: remote-http-tool
    mcp:
      host: http://localhost:9090/mcp
  # 方式 3: openapi 自动生成 MCP Tools (将 RESTful API 转为 MCP 工具)
  - name: petstore-api
    openapi:
      host: localhost:8081
      schema:
        file: ./openapi.json
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        let mcp = config.mcp.as_ref().expect("mcp section exists");
        assert_eq!(mcp.default_port, 6030);
        assert_eq!(mcp.session_mode, McpSessionMode::Stateful);
        assert_eq!(mcp.prefix_policy, McpPrefixPolicy::Always);
        assert_eq!(mcp.failure_policy, McpFailurePolicy::FailOpen);
        assert_eq!(mcp.policies.len(), 0);
        assert_eq!(mcp.targets.len(), 3);

        // Target 1: stdio
        let t1 = &mcp.targets[0];
        assert_eq!(t1.name, "time-tool");
        assert_eq!(t1.target_kind_str(), "stdio");
        let stdio = t1.stdio.as_ref().unwrap();
        assert_eq!(stdio.cmd, "uvx");
        assert_eq!(stdio.args, vec!["mcp-server-time"]);
        assert_eq!(stdio.env.get("TZ"), Some(&"UTC".to_string()));
        assert!(!stdio.clear_env);
        assert_eq!(
            t1.format_tool_name(mcp.prefix_policy, mcp.targets.len(), "get_time"),
            "time-tool__get_time"
        );

        // Target 2: mcp
        let t2 = &mcp.targets[1];
        assert_eq!(t2.name, "remote-http-tool");
        assert_eq!(t2.target_kind_str(), "mcp");
        let http = t2.mcp.as_ref().unwrap();
        assert_eq!(http.host, "http://localhost:9090/mcp");

        // Target 3: openapi
        let t3 = &mcp.targets[2];
        assert_eq!(t3.name, "petstore-api");
        assert_eq!(t3.target_kind_str(), "openapi");
        let openapi = t3.openapi.as_ref().unwrap();
        assert_eq!(openapi.host, "localhost:8081");
        assert_eq!(openapi.schema.file, Some("./openapi.json".to_string()));

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deserialize_llm_config() {
        let yaml = r#"
llm:
  defaultPort: 6020
  models:
  - subscribe: OpenAI
    id: 1-hash
    name: 123345@gmail.com
  - subscribe: OpenAI
    id: 2-hash
    name: 123345@cloud.com
    modelIgnore: 
    - gpt-5
    - gpt-5.5-mini
    - gpt-image-2
    modelAlias:
    - src: gpt-5
      cur: gpt5-high
    - src: gpt-4o
      cur: gpt4-medium
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        let llm = config.llm.as_ref().expect("llm section exists");
        assert_eq!(llm.default_port, 6020);
        assert_eq!(llm.subscriptions().len(), 2);

        let sub1 = llm.subscriptions()[0];
        assert_eq!(sub1.subscribe, "OpenAI");
        assert_eq!(sub1.id, "1-hash");
        assert_eq!(sub1.name, "123345@gmail.com");
        assert!(sub1.model_ignore.is_empty());
        assert!(sub1.model_alias.is_empty());

        let sub2 = llm.subscriptions()[1];
        assert_eq!(sub2.subscribe, "OpenAI");
        assert_eq!(sub2.id, "2-hash");
        assert_eq!(sub2.name, "123345@cloud.com");
        assert_eq!(
            sub2.model_ignore,
            vec!["gpt-5", "gpt-5.5-mini", "gpt-image-2"]
        );
        assert!(sub2.is_model_ignored("gpt-5"));
        assert!(sub2.is_model_ignored("GPT-5.5-MINI"));
        assert!(!sub2.is_model_ignored("gpt-4o"));

        assert_eq!(sub2.resolve_alias("gpt-5"), Some("gpt5-high"));
        assert_eq!(sub2.resolve_alias("gpt-4o"), Some("gpt4-medium"));
        assert_eq!(sub2.resolve_alias("gpt-3.5-turbo"), None);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deserialize_yaml_ports_config() {
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
    }
}
