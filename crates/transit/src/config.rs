use crate::{Result, TransitError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Default)]
pub struct MatchInput<'a> {
    pub host: &'a str,
    pub path: &'a str,
    pub headers: &'a [(String, String)],
    pub method: Option<&'a str>,
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
pub const UI_DEFAULT_PORT: u16 = 6000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiConfig>,
    #[serde(
        default,
        deserialize_with = "deserialize_routes_config",
        skip_serializing_if = "Option::is_none"
    )]
    pub routes: Option<RoutesConfig>,
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

fn deserialize_routes_config<'de, D>(
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

fn deserialize_route_listeners<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<RouteListenerConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    parse_listeners_from_value(value).map_err(serde::de::Error::custom)
}

fn parse_listeners_from_value(
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
pub struct UiConfig {
    #[serde(default = "default_ui_port")]
    pub port: u16,
}

fn default_ui_port() -> u16 {
    UI_DEFAULT_PORT
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    #[serde(default = "default_llm_port")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<LlmSubscriptionConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<LlmProviderConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<LlmModelItem>,
}

fn default_llm_port() -> u16 {
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

pub type LlmModelSubscription = LlmSubscriptionConfig;

impl LlmSubscriptionConfig {
    pub fn is_model_excluded(&self, model: &str) -> bool {
        self.excluded_models
            .iter()
            .any(|pattern| pattern.eq_ignore_ascii_case(model))
    }

    pub fn is_model_ignored(&self, model: &str) -> bool {
        self.is_model_excluded(model)
    }

    pub fn resolve_alias<'a>(&'a self, requested: &'a str) -> Option<&'a str> {
        self.model_aliases
            .iter()
            .find(|alias| alias.model.eq_ignore_ascii_case(requested) || alias.src().eq_ignore_ascii_case(requested))
            .map(|alias| alias.alias.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelAliasConfig {
    #[serde(alias = "src")]
    pub model: String,
    #[serde(alias = "cur")]
    pub alias: String,
}

impl ModelAliasConfig {
    pub fn src(&self) -> &str {
        &self.model
    }

    pub fn cur(&self) -> &str {
        &self.alias
    }
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

fn default_listener_name() -> String {
    "default".to_string()
}

fn default_listener_port() -> u16 {
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

fn default_target_name() -> String {
    "default-target".to_string()
}

fn deserialize_methods_option<'de, D>(
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

fn deserialize_route_policies<'de, D>(
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

fn deserialize_route_matches<'de, D>(
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

fn parse_matches_from_value(
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

fn parse_single_match_value(
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

impl RuntimeConfig {
    pub fn empty(version: impl Into<String>) -> Self {
        Self {
            version: Some(version.into()),
            llm: None,
            mcp: None,
            ui: None,
            routes: None,
        }
    }

    pub fn listeners(&self) -> &[RouteListenerConfig] {
        self.routes
            .as_ref()
            .map(|r| r.listeners.as_slice())
            .unwrap_or(&[])
    }

    pub fn validate(&self) -> std::result::Result<(), Vec<ConfigConflict>> {
        let mut conflicts = Vec::new();
        let mut ports_seen = BTreeSet::new();

        if let Some(llm) = &self.llm {
            ports_seen.insert(llm.port);
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
            if !ports_seen.insert(mcp.port) {
                conflicts.push(ConfigConflict::new(
                    "port-conflict",
                    format!(
                        "mcp port {} conflicts with another configured port",
                        mcp.port
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
                        "multiple-mcp-target-kinds",
                        format!(
                            "mcp target {} must specify only one of stdio, mcp, or openapi",
                            target.name
                        ),
                    ));
                }
            }
        }

        if let Some(ui) = &self.ui {
            if !ports_seen.insert(ui.port) {
                conflicts.push(ConfigConflict::new(
                    "port-conflict",
                    format!(
                        "ui port {} conflicts with another configured port",
                        ui.port
                    ),
                ));
            }
        }

        let mut listener_names = BTreeSet::new();
        for listener in self.listeners() {
            let l_port = listener.port();
            if !ports_seen.insert(l_port) {
                conflicts.push(ConfigConflict::new(
                    "duplicate-port",
                    format!("route listener port {} is defined more than once", l_port),
                ));
            }

            if !listener_names.insert(listener.name.clone()) {
                conflicts.push(ConfigConflict::new(
                    "duplicate-listener",
                    format!("route listener {} is defined more than once", listener.name),
                ));
            }
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(conflicts)
        }
    }

    pub fn route_for(&self, port: u16, input: &MatchInput<'_>) -> Result<&RouteTargetConfig> {
        // First match on exact port if specified and hostname
        for listener in self.listeners() {
            let l_port = listener.port();
            if l_port == port || port == 0 {
                if let Some(h) = &listener.hostname {
                    if !matches_hostname(h, input.host) {
                        continue;
                    }
                }
                for target in &listener.targets {
                    if target.matches(input) {
                        return Ok(target);
                    }
                }
            }
        }

        // Fallback: match across all listeners
        for listener in self.listeners() {
            if let Some(h) = &listener.hostname {
                if !matches_hostname(h, input.host) {
                    continue;
                }
            }
            for target in &listener.targets {
                if target.matches(input) {
                    return Ok(target);
                }
            }
        }

        Err(TransitError::RouteNotFound {
            host: input.host.to_string(),
            path: input.path.to_string(),
        })
    }
}

fn matches_hostname(pattern: &str, host: &str) -> bool {
    let host = host
        .split(':')
        .next()
        .unwrap_or(host)
        .trim()
        .to_ascii_lowercase();
    let pattern = pattern.trim().to_ascii_lowercase();
    if pattern == "*" || pattern.is_empty() || host == "*" || host.is_empty() {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        if host.ends_with(suffix) || host == suffix {
            return true;
        }
    }
    pattern == host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_llm_provider_and_models() {
        let yaml = r#"
llm:
  port: 6020
  models:
  - provider: openAI
    name: gpt-4o
    models:
    - name: gpt-4o
      apiKey: $OPENAI_API_KEY
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        let llm = config.llm.as_ref().expect("llm section exists");
        assert_eq!(llm.port, 6020);
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
routes:
  listeners:
    - name: default
      port: 6010
      protocol: http
      targets:
      - name: a2a-agent-route
        matches:
        - path:
            prefix: /a2a
        policies:
          a2a: {}
        backends:
        - host: localhost:9999
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.listeners().len(), 1);
        let listener = &config.listeners()[0];
        assert_eq!(listener.port(), 6010);
        assert_eq!(listener.name, "default");
        assert_eq!(listener.protocol, ListenerProtocol::Http);
        assert_eq!(listener.targets.len(), 1);

        let target = &listener.targets[0];
        assert_eq!(target.name, "a2a-agent-route");
        assert!(target.is_a2a_enabled());
        assert_eq!(target.backends.len(), 1);
        assert_eq!(target.target_hosts(), vec!["localhost:9999"]);

        let input_not_matched = MatchInput {
            host: "localhost",
            path: "/chat/completions",
            headers: &[],
            method: None,
        };
        assert!(!target.matches(&input_not_matched));

        let input_a2a = MatchInput {
            host: "localhost",
            path: "/a2a/v1/message",
            headers: &[],
            method: None,
        };
        assert!(target.matches(&input_a2a));

        let found = config.route_for(6010, &input_a2a).expect("found a2a route");
        assert_eq!(found.name, "a2a-agent-route");
    }

    #[test]
    fn test_deserialize_mcp_config() {
        let yaml = r#"
mcp:
  port: 6030
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
        assert_eq!(mcp.port, 6030);
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
  port: 6020
  subscriptions:
  - provider: OpenAI
    id: 1-hash
    account: 123345@gmail.com
  - provider: OpenAI
    id: 2-hash
    account: 123345@cloud.com
    excludedModels: 
    - gpt-5
    - gpt-5.5-mini
    - gpt-image-2
    modelAliases:
    - model: gpt-5
      alias: gpt5-high
    - model: gpt-4o
      alias: gpt4-medium
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        let llm = config.llm.as_ref().expect("llm section exists");
        assert_eq!(llm.port, 6020);
        assert_eq!(llm.subscriptions().len(), 2);

        let sub1 = llm.subscriptions()[0];
        assert_eq!(sub1.provider, "OpenAI");
        assert_eq!(sub1.id, "1-hash");
        assert_eq!(sub1.account, "123345@gmail.com");
        assert!(sub1.excluded_models.is_empty());
        assert!(sub1.model_aliases.is_empty());

        let sub2 = llm.subscriptions()[1];
        assert_eq!(sub2.provider, "OpenAI");
        assert_eq!(sub2.id, "2-hash");
        assert_eq!(sub2.account, "123345@cloud.com");
        assert_eq!(
            sub2.excluded_models,
            vec!["gpt-5", "gpt-5.5-mini", "gpt-image-2"]
        );
        assert!(sub2.is_model_excluded("gpt-5"));
        assert!(sub2.is_model_ignored("GPT-5.5-MINI"));
        assert!(!sub2.is_model_excluded("gpt-4o"));

        assert_eq!(sub2.resolve_alias("gpt-5"), Some("gpt5-high"));
        assert_eq!(sub2.resolve_alias("gpt-4o"), Some("gpt4-medium"));
        assert_eq!(sub2.resolve_alias("gpt-3.5-turbo"), None);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deserialize_yaml_routes_config() {
        let yaml = r#"
routes:
  listeners:
    - name: default
      port: 6000
      protocol: http
      targets:
        - name: api-route
          matches:
            - path:
                prefix: /api
          backends:
            - 127.0.0.1:8080
        - name: a2a-agent-route
          matches:
            - path:
                prefix: /a2a
          policies:
            a2a: {}
          backends:
            - 127.0.0.1:9999
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.listeners().len(), 1);
        let listener = &config.listeners()[0];
        assert_eq!(listener.name, "default");
        assert_eq!(listener.port, 6000);
        assert_eq!(listener.protocol, ListenerProtocol::Http);
        assert_eq!(listener.targets.len(), 2);

        let target1 = &listener.targets[0];
        assert_eq!(target1.name, "api-route");
        assert_eq!(target1.target_hosts(), vec!["127.0.0.1:8080"]);
        assert!(!target1.is_a2a_enabled());

        let target2 = &listener.targets[1];
        assert_eq!(target2.name, "a2a-agent-route");
        assert_eq!(target2.target_hosts(), vec!["127.0.0.1:9999"]);
        assert!(target2.is_a2a_enabled());

        let input_matched = MatchInput {
            host: "localhost",
            path: "/api/v1/users",
            headers: &[],
            method: None,
        };
        assert!(target1.matches(&input_matched));

        let route_found = config.route_for(6000, &input_matched).expect("route_for");
        assert_eq!(route_found.name, "api-route");

        let input_a2a = MatchInput {
            host: "localhost",
            path: "/a2a/v1/message",
            headers: &[],
            method: None,
        };
        assert!(target2.matches(&input_a2a));
        let route_a2a_found = config.route_for(6000, &input_a2a).expect("route_for a2a");
        assert_eq!(route_a2a_found.name, "a2a-agent-route");
    }

    #[test]
    fn test_deserialize_ui_config() {
        let yaml = r#"
ui:
  port: 6000
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        let ui = config.ui.as_ref().expect("ui section exists");
        assert_eq!(ui.port, 6000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deserialize_port_route_hostname_and_tls() {
        let yaml = r#"
routes:
  listeners:
    - name: https-route
      port: 443
      protocol: https
      hostname: "*.example.com"
      tls:
        certificate: /path/to/cert.pem
        privateKey: /path/to/key.pem
      targets:
        - name: secure-rule
          matches:
            - path:
                prefix: /secure
          backends:
            - host: localhost:8443
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.listeners().len(), 1);
        let listener = &config.listeners()[0];
        assert_eq!(listener.port(), 443);
        assert_eq!(listener.name, "https-route");
        assert_eq!(listener.protocol, ListenerProtocol::Https);
        assert_eq!(listener.hostname, Some("*.example.com".to_string()));
        assert!(listener.is_tls());

        let tls = listener.tls.as_ref().expect("tls config exists");
        assert_eq!(tls.certificate, Some("/path/to/cert.pem".to_string()));
        assert_eq!(tls.private_key, Some("/path/to/key.pem".to_string()));
    }

    #[test]
    fn test_deserialize_rewritten_routes_with_listeners_and_targets() {
        let yaml = r#"
routes:
  listeners:
    - name: default
      port: 6000
      protocol: http
      targets:
        - name: api-route
          matches:
            - path:
                prefix: /api
          backends:
            - 127.0.0.1:8080
        - name: a2a-agent-route
          matches:
            - path:
                prefix: /a2a
          policies:
            a2a: {}
          backends:
            - 127.0.0.1:9999
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.listeners().len(), 1);
        let listener = &config.listeners()[0];
        assert_eq!(listener.name, "default");
        assert_eq!(listener.port, 6000);
        assert_eq!(listener.protocol, ListenerProtocol::Http);
        assert_eq!(listener.targets.len(), 2);

        let target1 = &listener.targets[0];
        assert_eq!(target1.name, "api-route");
        assert_eq!(target1.target_hosts(), vec!["127.0.0.1:8080"]);

        let target2 = &listener.targets[1];
        assert_eq!(target2.name, "a2a-agent-route");
        assert!(target2.is_a2a_enabled());
        assert_eq!(target2.target_hosts(), vec!["127.0.0.1:9999"]);

        let input_api = MatchInput {
            host: "127.0.0.1",
            path: "/api/test",
            headers: &[],
            method: None,
        };
        let route_api = config.route_for(6000, &input_api).expect("route api");
        assert_eq!(route_api.name, "api-route");

        let input_a2a = MatchInput {
            host: "127.0.0.1",
            path: "/a2a/v1",
            headers: &[],
            method: None,
        };
        let route_a2a = config.route_for(6000, &input_a2a).expect("route a2a");
        assert_eq!(route_a2a.name, "a2a-agent-route");
    }

    #[test]
    fn test_deserialize_flat_routes_with_method_and_path_prefix() {
        let yaml = r#"
routes:
  - hostname: "*.example.com"
  - name: api
    port: 6010
    method: GET
    matches:
      pathPrefix: /api
    backends:
      - 127.0.0.1:8080
      - localhost:8000
    policies: []

  - name: a2a-agent
    port: 6666
    method: POST
    matches:
      pathPrefix: /a2a
    backends:
      - 127.0.0.1:9999
      - localhost:8080
    policies:
      a2a: {}
"#;

        let config: RuntimeConfig = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(config.listeners().len(), 2);

        let listener_api = &config.listeners()[0];
        assert_eq!(listener_api.name, "api");
        assert_eq!(listener_api.port, 6010);
        assert_eq!(listener_api.hostname, Some("*.example.com".to_string()));
        assert_eq!(listener_api.targets.len(), 1);
        let target_api = &listener_api.targets[0];
        assert_eq!(target_api.name, "api");
        assert_eq!(target_api.methods, Some(vec!["GET".to_string()]));
        assert_eq!(
            target_api.target_hosts(),
            vec!["127.0.0.1:8080", "localhost:8000"]
        );

        let listener_a2a = &config.listeners()[1];
        assert_eq!(listener_a2a.name, "a2a-agent");
        assert_eq!(listener_a2a.port, 6666);
        assert_eq!(listener_a2a.hostname, Some("*.example.com".to_string()));
        assert_eq!(listener_a2a.targets.len(), 1);
        let target_a2a = &listener_a2a.targets[0];
        assert_eq!(target_a2a.name, "a2a-agent");
        assert_eq!(target_a2a.methods, Some(vec!["POST".to_string()]));
        assert!(target_a2a.is_a2a_enabled());
        assert_eq!(
            target_a2a.target_hosts(),
            vec!["127.0.0.1:9999", "localhost:8080"]
        );

        // Test route matching with methods
        let input_get_api = MatchInput {
            host: "api.example.com",
            path: "/api/v1/resource",
            headers: &[],
            method: Some("GET"),
        };
        let found = config.route_for(6010, &input_get_api).expect("found api GET");
        assert_eq!(found.name, "api");

        let input_post_api = MatchInput {
            host: "api.example.com",
            path: "/api/v1/resource",
            headers: &[],
            method: Some("POST"),
        };
        assert!(config.route_for(6010, &input_post_api).is_err());

        let input_post_a2a = MatchInput {
            host: "agent.example.com",
            path: "/a2a/v1/agent",
            headers: &[],
            method: Some("POST"),
        };
        let found_a2a = config.route_for(6666, &input_post_a2a).expect("found a2a POST");
        assert_eq!(found_a2a.name, "a2a-agent");
    }
}
