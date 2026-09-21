use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const LLM_DEFAULT_PORT: u16 = 6020;
pub const MCP_DEFAULT_PORT: u16 = 6030;
pub const HTTP_LISTENER_PORT: u16 = 6010;
pub const HTTPS_LISTENER_PORT: u16 = 26443;
pub const UI_PORT: u16 = 26021;
pub const UI_DEFAULT_PORT: u16 = 6010;

#[derive(Debug, Clone, Copy, Default)]
pub struct MatchInput<'a> {
    pub host: &'a str,
    pub path: &'a str,
    pub headers: &'a [(String, String)],
    pub method: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpConfig>,
    #[serde(
        default,
        deserialize_with = "deserialize_routes_config",
        skip_serializing_if = "Option::is_none"
    )]
    pub routes: Option<RoutesConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binds: Vec<RouteListenerConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backends: Vec<RouteBackend>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policies: Option<RoutePolicies>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    #[serde(default = "default_ui_port")]
    pub port: u16,
}

pub fn default_ui_port() -> u16 {
    UI_DEFAULT_PORT
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    #[serde(default = "default_llm_port")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<LlmSubscriptionConfig>,
    #[serde(default, alias = "platforms", skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<LlmProviderConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<LlmModelItem>,
}

pub fn default_llm_port() -> u16 {
    LLM_DEFAULT_PORT
}

impl LlmConfig {
    pub fn subscriptions(&self) -> Vec<&LlmSubscriptionConfig> {
        let mut list: Vec<&LlmSubscriptionConfig> = self.subscriptions.iter().collect();
        for item in &self.models {
            if let LlmModelItem::Subscription(sub) = item {
                list.push(sub);
            }
        }
        list
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

    pub fn platforms(&self) -> Vec<&LlmProviderConfig> {
        self.providers()
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
    Subscription(LlmSubscriptionConfig),
    Provider(LlmProviderConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderConfig {
    #[serde(alias = "platform")]
    pub provider: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<LlmProviderModel>,
}

pub type LlmPlatformConfig = LlmProviderConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LlmPlatformModel {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

pub type LlmProviderModel = LlmPlatformModel;

impl LlmPlatformModel {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct LlmSubscriptionConfig {
    #[serde(alias = "subscribe")]
    pub provider: String,
    pub id: String,
    #[serde(alias = "name")]
    pub account: String,
    #[serde(default, alias = "modelIgnore", skip_serializing_if = "Vec::is_empty")]
    pub excluded_models: Vec<String>,
    #[serde(default, alias = "modelAlias", skip_serializing_if = "Vec::is_empty")]
    pub model_aliases: Vec<ModelAliasConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelAliasConfig {
    pub model: String,
    pub alias: String,
}

pub type ModelAlias = ModelAliasConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    #[serde(default = "default_mcp_port")]
    pub port: u16,
    #[serde(default)]
    pub session_mode: McpSessionMode,
    #[serde(default)]
    pub prefix_policy: McpPrefixPolicy,
    #[serde(default)]
    pub failure_policy: McpFailurePolicy,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<serde_json::Value>,
    #[serde(default)]
    pub targets: Vec<McpTargetConfig>,
}

pub fn default_mcp_port() -> u16 {
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
pub struct McpTargetConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdio: Option<McpStdioTargetConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpHttpTargetConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openapi: Option<McpOpenApiTargetConfig>,
}

impl McpTargetConfig {
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
        target_count: usize,
        tool_name: &str,
    ) -> String {
        let should_prefix = match prefix_policy {
            McpPrefixPolicy::Always => true,
            McpPrefixPolicy::Conditional => target_count > 1,
        };

        if should_prefix {
            format!("{}__{}", self.name, tool_name)
        } else {
            tool_name.to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpStdioTargetConfig {
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
pub struct McpHttpTargetConfig {
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpOpenApiTargetConfig {
    pub host: String,
    pub schema: McpOpenApiSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpOpenApiSchema {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutesConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_route_listeners",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub listeners: Vec<RouteListenerConfig>,
}

pub fn deserialize_routes_config<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<RoutesConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = match Option::<serde_json::Value>::deserialize(deserializer)? {
        Some(v) => v,
        None => return Ok(None),
    };

    let listeners = parse_listeners_from_value(value).map_err(serde::de::Error::custom)?;
    Ok(Some(RoutesConfig { listeners }))
}

pub fn deserialize_route_listeners<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<RouteListenerConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    parse_listeners_from_value(value).map_err(serde::de::Error::custom)
}

pub fn parse_listeners_from_value(
    value: serde_json::Value,
) -> std::result::Result<Vec<RouteListenerConfig>, String> {
    match value {
        serde_json::Value::Object(mut map) => {
            if let Some(listeners_val) = map.remove("listeners") {
                parse_listeners_from_value(listeners_val)
            } else if map.contains_key("targets") || map.contains_key("rules") {
                let listener: RouteListenerConfig =
                    serde_json::from_value(serde_json::Value::Object(map))
                        .map_err(|e| e.to_string())?;
                Ok(vec![listener])
            } else if map.contains_key("matches")
                || map.contains_key("backends")
                || map.contains_key("method")
                || map.contains_key("methods")
                || map.contains_key("policies")
            {
                let port = map
                    .get("port")
                    .and_then(|p| p.as_u64())
                    .map(|p| p as u16)
                    .unwrap_or(HTTP_LISTENER_PORT);
                let protocol: ListenerProtocol = map
                    .get("protocol")
                    .and_then(|p| serde_json::from_value(p.clone()).ok())
                    .unwrap_or_default();
                let hostname = map
                    .get("hostname")
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string());
                let tls: Option<TlsConfig> = map
                    .get("tls")
                    .and_then(|t| serde_json::from_value(t.clone()).ok());
                let target: RouteTargetConfig =
                    serde_json::from_value(serde_json::Value::Object(map))
                        .map_err(|e| e.to_string())?;
                let listener_name = if target.name.is_empty() || target.name == "default-target" {
                    format!("listener-{}", port)
                } else {
                    target.name.clone()
                };
                Ok(vec![RouteListenerConfig {
                    name: listener_name,
                    port,
                    protocol,
                    hostname,
                    tls,
                    targets: vec![target],
                }])
            } else {
                let mut listeners = Vec::new();
                for (key, val) in map {
                    let mut sub_listeners = parse_listeners_from_value(val)?;
                    for l in &mut sub_listeners {
                        if l.name == "default" && key != "default" {
                            l.name = key.clone();
                        }
                    }
                    listeners.extend(sub_listeners);
                }
                Ok(listeners)
            }
        }
        serde_json::Value::Array(items) => {
            let mut ambient_hostname = None;
            for item in &items {
                if let serde_json::Value::Object(map) = item {
                    if map.len() == 1 && map.contains_key("hostname") {
                        if let Some(h) = map.get("hostname").and_then(|h| h.as_str()) {
                            ambient_hostname = Some(h.to_string());
                        }
                    }
                }
            }

            let mut listeners: Vec<RouteListenerConfig> = Vec::new();
            for item in items {
                if let serde_json::Value::Object(ref map) = item {
                    if map.len() == 1 && map.contains_key("hostname") {
                        continue;
                    }
                    if map.contains_key("targets") || map.contains_key("rules") {
                        let mut listener: RouteListenerConfig =
                            serde_json::from_value(item).map_err(|e| e.to_string())?;
                        if listener.hostname.is_none() {
                            listener.hostname = ambient_hostname.clone();
                        }
                        listeners.push(listener);
                        continue;
                    }
                    let port = map
                        .get("port")
                        .and_then(|p| p.as_u64())
                        .map(|p| p as u16)
                        .unwrap_or(HTTP_LISTENER_PORT);
                    let protocol: ListenerProtocol = map
                        .get("protocol")
                        .and_then(|p| serde_json::from_value(p.clone()).ok())
                        .unwrap_or_default();
                    let hostname = map
                        .get("hostname")
                        .and_then(|h| h.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| ambient_hostname.clone());
                    let tls: Option<TlsConfig> = map
                        .get("tls")
                        .and_then(|t| serde_json::from_value(t.clone()).ok());
                    let target: RouteTargetConfig =
                        serde_json::from_value(item).map_err(|e| e.to_string())?;

                    if let Some(existing) = listeners.iter_mut().find(|l| {
                        l.port == port
                            && l.protocol == protocol
                            && l.hostname == hostname
                            && l.tls == tls
                    }) {
                        existing.targets.push(target);
                    } else {
                        let listener_name =
                            if target.name.is_empty() || target.name == "default-target" {
                                format!("listener-{}", port)
                            } else {
                                target.name.clone()
                            };
                        listeners.push(RouteListenerConfig {
                            name: listener_name,
                            port,
                            protocol,
                            hostname,
                            tls,
                            targets: vec![target],
                        });
                    }
                }
            }
            Ok(listeners)
        }
        serde_json::Value::Null => Ok(Vec::new()),
        _ => Err("Invalid routes configuration format".to_string()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteListenerConfig {
    #[serde(default = "default_listener_name")]
    pub name: String,
    #[serde(default = "default_listener_port")]
    pub port: u16,
    #[serde(default)]
    pub protocol: ListenerProtocol,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    #[serde(
        default,
        alias = "rules",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub targets: Vec<RouteTargetConfig>,
}

pub fn default_listener_name() -> String {
    "default".to_string()
}

pub fn default_listener_port() -> u16 {
    HTTP_LISTENER_PORT
}

pub type ListenerConfig = RouteListenerConfig;
pub type RouteConfig = RouteListenerConfig;
pub type RouteRule = RouteTargetConfig;
pub type RouteRuleConfig = RouteTargetConfig;
pub type RouteProtocol = ListenerProtocol;

impl RouteListenerConfig {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_tls(&self) -> bool {
        self.tls.is_some() || matches!(self.protocol, ListenerProtocol::Https | ListenerProtocol::Tls)
    }

    pub fn effective_port(&self, fallback: u16) -> u16 {
        if self.port != 0 {
            self.port
        } else {
            fallback
        }
    }

    pub fn targets(&self) -> &[RouteTargetConfig] {
        &self.targets
    }

    pub fn rules(&self) -> &[RouteTargetConfig] {
        &self.targets
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct TlsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
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
pub struct RouteTargetConfig {
    #[serde(default = "default_target_name")]
    pub name: String,
    #[serde(
        default,
        alias = "method",
        deserialize_with = "deserialize_methods_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub methods: Option<Vec<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_route_matches",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub matches: Vec<RouteMatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a2a: Option<A2aConfig>,
    #[serde(
        default,
        deserialize_with = "deserialize_route_policies",
        skip_serializing_if = "Option::is_none"
    )]
    pub policies: Option<RoutePolicies>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endpoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backends: Vec<RouteBackend>,
}

pub fn default_target_name() -> String {
    "default-target".to_string()
}

pub fn deserialize_methods_option<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MethodInput {
        One(String),
        Many(Vec<String>),
    }

    match Option::<MethodInput>::deserialize(deserializer)? {
        Some(MethodInput::One(s)) => Ok(Some(vec![s])),
        Some(MethodInput::Many(v)) => Ok(Some(v)),
        None => Ok(None),
    }
}

pub type RouteRuleConfigAlias = RouteTargetConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutePolicies {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a2a: Option<A2aConfig>,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

pub fn deserialize_route_policies<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<RoutePolicies>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PoliciesInput {
        Object(RoutePolicies),
        Array(Vec<serde_json::Value>),
    }

    match Option::<PoliciesInput>::deserialize(deserializer)? {
        Some(PoliciesInput::Object(obj)) => Ok(Some(obj)),
        Some(PoliciesInput::Array(arr)) => {
            let mut policies = RoutePolicies::default();
            for item in arr {
                if let serde_json::Value::Object(map) = item {
                    if let Some(a2a_val) = map.get("a2a") {
                        if let Ok(a2a_cfg) = serde_json::from_value::<A2aConfig>(a2a_val.clone()) {
                            policies.a2a = Some(a2a_cfg);
                        }
                    }
                    for (k, v) in map {
                        if k != "a2a" {
                            policies.extra.insert(k, v);
                        }
                    }
                }
            }
            Ok(Some(policies))
        }
        None => Ok(None),
    }
}

impl RouteTargetConfig {
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
        self.a2a.is_some() || self.policies.as_ref().and_then(|p| p.a2a.as_ref()).is_some()
    }

    pub fn matches(&self, input: &MatchInput<'_>) -> bool {
        if let Some(methods) = &self.methods {
            if !methods.is_empty() {
                if let Some(m) = input.method {
                    if !methods.iter().any(|item| item.eq_ignore_ascii_case(m)) {
                        return false;
                    }
                }
            }
        }

        if self.matches.is_empty() {
            return true;
        }
        self.matches.iter().any(|m| m.matches(input))
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
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "method")]
    pub methods: Option<Vec<String>>,
}

pub fn deserialize_route_matches<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<RouteMatch>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = match Option::<serde_json::Value>::deserialize(deserializer)? {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    parse_matches_from_value(value).map_err(serde::de::Error::custom)
}

pub fn parse_matches_from_value(
    value: serde_json::Value,
) -> std::result::Result<Vec<RouteMatch>, String> {
    match value {
        serde_json::Value::Array(arr) => {
            let mut res = Vec::new();
            for item in arr {
                res.extend(parse_single_match_value(item)?);
            }
            Ok(res)
        }
        other => parse_single_match_value(other),
    }
}

pub fn parse_single_match_value(
    value: serde_json::Value,
) -> std::result::Result<Vec<RouteMatch>, String> {
    match value {
        serde_json::Value::String(s) => Ok(vec![RouteMatch {
            path: Some(PathMatch {
                prefix: Some(s),
                exact: None,
                regex: None,
            }),
            headers: None,
            methods: None,
        }]),
        serde_json::Value::Object(map) => {
            let mut prefix = None;
            let mut exact = None;
            let mut regex = None;

            if let Some(p) = map
                .get("pathPrefix")
                .or_else(|| map.get("path_prefix"))
                .or_else(|| map.get("prefix"))
            {
                if let Some(s) = p.as_str() {
                    prefix = Some(s.to_string());
                }
            }
            if let Some(e) = map
                .get("pathExact")
                .or_else(|| map.get("path_exact"))
                .or_else(|| map.get("exact"))
            {
                if let Some(s) = e.as_str() {
                    exact = Some(s.to_string());
                }
            }
            if let Some(r) = map
                .get("pathRegex")
                .or_else(|| map.get("path_regex"))
                .or_else(|| map.get("regex"))
            {
                if let Some(s) = r.as_str() {
                    regex = Some(s.to_string());
                }
            }

            if let Some(path_obj) = map.get("path") {
                if let Ok(pm) = serde_json::from_value::<PathMatch>(path_obj.clone()) {
                    if pm.prefix.is_some() {
                        prefix = pm.prefix;
                    }
                    if pm.exact.is_some() {
                        exact = pm.exact;
                    }
                    if pm.regex.is_some() {
                        regex = pm.regex;
                    }
                } else if let Some(s) = path_obj.as_str() {
                    prefix = Some(s.to_string());
                }
            }

            let path = if prefix.is_some() || exact.is_some() || regex.is_some() {
                Some(PathMatch {
                    prefix,
                    exact,
                    regex,
                })
            } else {
                None
            };

            let headers: Option<Vec<HeaderMatch>> = map
                .get("headers")
                .and_then(|h| serde_json::from_value(h.clone()).ok());

            let mut methods: Option<Vec<String>> = None;
            if let Some(m) = map.get("methods").or_else(|| map.get("method")) {
                if let Some(s) = m.as_str() {
                    methods = Some(vec![s.to_string()]);
                } else if let Ok(v) = serde_json::from_value::<Vec<String>>(m.clone()) {
                    methods = Some(v);
                }
            }

            Ok(vec![RouteMatch {
                path,
                headers,
                methods,
            }])
        }
        serde_json::Value::Null => Ok(Vec::new()),
        _ => Err("Invalid match format".to_string()),
    }
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
        if let Some(methods) = &self.methods {
            if !methods.is_empty() {
                if let Some(m) = input.method {
                    if !methods.iter().any(|item| item.eq_ignore_ascii_case(m)) {
                        return false;
                    }
                }
            }
        }

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
