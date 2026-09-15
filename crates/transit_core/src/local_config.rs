use crate::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, path::Path, result::Result};

/// Transit accepts its full runtime format or the concise gateway format.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ConfigDocument {
    Runtime(RuntimeConfig),
    Gateway(GatewayConfig),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GatewayConfig {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gateways: BTreeMap<String, GatewayListener>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LocalLlmConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<LocalLlmRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GatewayListener {
    #[schemars(range(min = 1, max = 65535))]
    pub port: u16,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalLlmConfig {
    /// HTTP port for automatic model routing. Defaults to the first gateway port, or 4000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1, max = 65535))]
    pub port: Option<u16>,
    /// Named providers shared by the YAML configuration and Platform's provider selector.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<LocalLlmProvider>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<LocalLlmModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalLlmProvider {
    pub name: String,
    pub provider: LocalProviderKind,
    /// Environment reference, for example $OPENAI_API_KEY. Resolved at request time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^\$[A-Za-z_][A-Za-z0-9_]*$"))]
    pub api_key: Option<String>,
    #[serde(default, rename = "baseURL", alias = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<SecretKeyReference>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum LocalProviderKind {
    #[serde(rename = "openAI", alias = "openai", alias = "open-ai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "deepseek", alias = "deep-seek")]
    DeepSeek,
    #[serde(rename = "openai-compatible", alias = "open-ai-compatible")]
    OpenAiCompatible,
}

impl From<LocalProviderKind> for ProviderKind {
    fn from(value: LocalProviderKind) -> Self {
        match value {
            LocalProviderKind::OpenAi => Self::OpenAi,
            LocalProviderKind::Anthropic => Self::Anthropic,
            LocalProviderKind::Gemini => Self::Gemini,
            LocalProviderKind::DeepSeek => Self::DeepSeek,
            LocalProviderKind::OpenAiCompatible => Self::OpenAiCompatible,
        }
    }
}

impl From<ProviderKind> for LocalProviderKind {
    fn from(value: ProviderKind) -> Self {
        match value {
            ProviderKind::OpenAi => Self::OpenAi,
            ProviderKind::Anthropic => Self::Anthropic,
            ProviderKind::Gemini => Self::Gemini,
            ProviderKind::DeepSeek => Self::DeepSeek,
            ProviderKind::OpenAiCompatible => Self::OpenAiCompatible,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalLlmModel {
    /// A name declared in llm.providers, or a built-in provider such as openAI.
    pub provider: String,
    /// Exact incoming model ID, or * for any model. Exact bindings take precedence.
    #[serde(rename = "type")]
    #[schemars(length(min = 1, max = 200))]
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^\$[A-Za-z_][A-Za-z0-9_]*$"))]
    pub api_key: Option<String>,
    #[serde(default, rename = "baseURL", alias = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalLlmRoute {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Omit to attach to all gateways; otherwise select a named gateway.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    pub backends: Vec<LocalAiBackend>,
    pub policies: LocalAiPolicies,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalAiBackend {
    pub ai: LocalAi,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalAi {
    pub provider: InlineAiProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InlineAiProvider {
    #[serde(rename = "type")]
    pub kind: LocalProviderKind,
    #[serde(default, rename = "openAI", skip_serializing_if = "Option::is_none")]
    pub openai: Option<InlineProviderOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anthropic: Option<InlineProviderOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini: Option<InlineProviderOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deepseek: Option<InlineProviderOptions>,
    #[serde(default, rename = "openai-compatible", skip_serializing_if = "Option::is_none")]
    pub compatible: Option<InlineProviderOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InlineProviderOptions {
    #[serde(default, rename = "baseURL", alias = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalAiPolicies {
    pub ai: LocalAiPaths,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_auth: Option<LocalBackendAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalAiPaths {
    /// Map incoming HTTP paths to upstream operations.
    pub routes: BTreeMap<String, AiOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocalBackendAuth {
    #[schemars(regex(pattern = r"^\$[A-Za-z_][A-Za-z0-9_]*$"))]
    pub key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AiOperation {
    Completions,
    Embeddings,
    Responses,
    Models,
}

impl AiOperation {
    fn path(self) -> &'static str {
        match self {
            Self::Completions => "/v1/chat/completions",
            Self::Embeddings => "/v1/embeddings",
            Self::Responses => "/v1/responses",
            Self::Models => "/v1/models",
        }
    }
}

fn env_key(value: Option<&str>) -> Result<Option<String>, String> {
    value.map(|key| {
        let name = key.strip_prefix('$').ok_or("apiKey must reference an environment variable, for example $OPENAI_API_KEY")?;
        if name.is_empty() || !name.bytes().enumerate().all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit())) {
            return Err("Invalid API key environment reference".into());
        }
        Ok(name.to_string())
    }).transpose()
}

impl LocalLlmProvider {
    fn compile(&self) -> Result<Provider, String> {
        if self.name.is_empty() || self.name.chars().any(char::is_control) {
            return Err("Provider name must not be empty or contain control characters".into());
        }
        if self.api_key.is_some() && self.credential_ref.is_some() {
            return Err(format!("Provider {} has both apiKey and credentialRef", self.name));
        }
        let provider = Provider {
            name: self.name.clone(), kind: self.provider.into(),
            base_url: self.base_url.clone().unwrap_or_default(),
            api_key_env: env_key(self.api_key.as_deref())?,
            credential_ref: self.credential_ref.clone(), request_headers: vec![],
        };
        if !(provider.effective_base_url().starts_with("http://") || provider.effective_base_url().starts_with("https://")) {
            return Err(format!("Provider {} requires an HTTP(S) baseURL", self.name));
        }
        Ok(provider)
    }
}

impl ConfigDocument {
    pub fn parse(path: &Path, raw: &[u8]) -> Result<Self, String> {
        let json = path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("json"));
        let value: serde_json::Value = if json {
            serde_json::from_slice(raw).map_err(|e| e.to_string())?
        } else {
            serde_yaml::from_slice(raw).map_err(|e| e.to_string())?
        };
        let concise = value.get("llm").is_some() || value.get("gateways").is_some()
            || value.get("routes").and_then(|r| r.as_array()).is_some_and(|r| r.iter().any(|r| r.get("backends").is_some()));
        if concise {
            serde_json::from_value(value).map(Self::Gateway).map_err(|e| e.to_string())
        } else {
            serde_json::from_value(value).map(Self::Runtime).map_err(|e| e.to_string())
        }
    }

    pub fn runtime(&self) -> Result<RuntimeConfig, String> {
        let config = match self {
            Self::Runtime(config) => config.clone(),
            Self::Gateway(config) => config.compile()?,
        };
        config.validate().map_err(|errors| format!("Invalid local configuration: {errors:?}"))?;
        Ok(config)
    }

    /// Keep the user's concise YAML structure when Platform adds providers or models.
    pub fn apply_platform_additions(&mut self, updated: &RuntimeConfig) -> Result<(), String> {
        let original = self.runtime()?;
        let Self::Gateway(document) = self else {
            *self = Self::Runtime(updated.clone());
            return Ok(());
        };
        document.version = updated.version.clone();
        let llm = document.llm.get_or_insert_with(LocalLlmConfig::default);
        for provider in &updated.providers {
            if original.provider(&provider.name).is_none() {
                llm.providers.push(LocalLlmProvider {
                    name: provider.name.clone(), provider: provider.kind.into(),
                    api_key: provider.api_key_env.as_ref().map(|key| format!("${key}")),
                    base_url: (!provider.base_url.is_empty()).then(|| provider.base_url.clone()),
                    credential_ref: provider.credential_ref.clone(),
                });
            }
        }
        for backend in &updated.backends {
            if original.backend(&backend.name).is_some() { continue; }
            if let BackendKind::Llm { provider, models, .. } = &backend.kind {
                for model in models {
                    llm.models.push(LocalLlmModel { provider: provider.clone(), model: model.clone(), api_key: None, base_url: None });
                }
            }
        }
        self.runtime()?;
        Ok(())
    }
}

impl GatewayConfig {
    fn compile(&self) -> Result<RuntimeConfig, String> {
        let version = if self.version.is_empty() {
            // Stable content version makes optimistic UI writes notice manual YAML edits.
            let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
            let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ *byte as u64).wrapping_mul(0x100000001b3));
            format!("local-{hash:016x}")
        } else { self.version.clone() };
        let mut config = RuntimeConfig::empty(version);
        for (name, gateway) in &self.gateways {
            if name.is_empty() { return Err("Gateway name must not be empty".into()); }
            add_listener(&mut config, name, gateway.port)?;
        }
        if !self.routes.is_empty() && config.listeners.is_empty() {
            add_listener(&mut config, "default", 4000)?;
        }
        for (index, route) in self.routes.iter().enumerate() {
            let name = route.name.clone().unwrap_or_else(|| format!("route-{index}"));
            let ports = if let Some(gateway) = &route.gateway {
                vec![self.gateways.get(gateway).ok_or_else(|| format!("Route {name} references missing gateway {gateway}"))?.port]
            } else { config.listeners.iter().map(|listener| listener.bind.port()).collect() };
            if route.backends.is_empty() || route.policies.ai.routes.is_empty() {
                return Err(format!("Route {name} requires backends and policies.ai.routes"));
            }
            let mut weighted = Vec::new();
            for (backend_index, backend) in route.backends.iter().enumerate() {
                let inline = &backend.ai.provider;
                let choices = [&inline.openai, &inline.anthropic, &inline.gemini, &inline.deepseek, &inline.compatible];
                let selected = match inline.kind {
                    LocalProviderKind::OpenAi => &inline.openai,
                    LocalProviderKind::Anthropic => &inline.anthropic,
                    LocalProviderKind::Gemini => &inline.gemini,
                    LocalProviderKind::DeepSeek => &inline.deepseek,
                    LocalProviderKind::OpenAiCompatible => &inline.compatible,
                };
                if choices.iter().filter(|c| c.is_some()).count() > usize::from(selected.is_some()) {
                    return Err(format!("Route {name}: provider options do not match type"));
                }
                let provider = LocalLlmProvider {
                    name: format!("{name}-provider-{backend_index}"), provider: inline.kind,
                    api_key: route.policies.backend_auth.as_ref().map(|auth| auth.key.clone()),
                    base_url: selected.as_ref().and_then(|options| options.base_url.clone()), credential_ref: None,
                }.compile()?;
                let backend_name = format!("{}/{}", provider.name, "*");
                config.backends.push(model_backend(&backend_name, &provider.name, "*"));
                config.providers.push(provider);
                weighted.push(WeightedBackend { name: backend_name, weight: 1, priority: None });
            }
            for (path, operation) in &route.policies.ai.routes {
                if !path.starts_with('/') || path.contains(['?', '#']) {
                    return Err(format!("Route {name} requires an absolute HTTP path without query or fragment"));
                }
                config.routes.push(AgentRoute {
                    name: format!("{name}:{path}"), protocol: AgentProtocol::Llm,
                    listener_ports: ports.clone(), matches: vec![model_match(PathMatch::Exact(path.clone()), None)],
                    weighted_backends: weighted.clone(), policies: vec![],
                    replace_prefix_match: (path != operation.path()).then(|| operation.path().to_string()),
                });
            }
        }
        if let Some(llm) = &self.llm {
            for provider in &llm.providers { config.providers.push(provider.compile()?); }
            let port = llm.port.unwrap_or_else(|| config.listeners.first().map(|l| l.bind.port()).unwrap_or(4000));
            for model in &llm.models {
                if config.provider(&model.provider).is_none() {
                    let kind: LocalProviderKind = serde_json::from_value(serde_json::Value::String(model.provider.clone()))
                        .map_err(|_| format!("Unknown provider {}; declare it in llm.providers", model.provider))?;
                    config.providers.push(LocalLlmProvider {
                        name: model.provider.clone(), provider: kind, api_key: model.api_key.clone(),
                        base_url: model.base_url.clone(), credential_ref: None,
                    }.compile()?);
                } else if model.api_key.is_some() || model.base_url.is_some() {
                    let provider = config.provider(&model.provider).unwrap();
                    if model.api_key.is_some() && env_key(model.api_key.as_deref())? != provider.api_key_env
                        || model.base_url.as_ref().is_some_and(|url| url != provider.effective_base_url()) {
                        return Err(format!("Conflicting settings for provider {}; declare separate named providers", model.provider));
                    }
                }
                add_model_binding(&mut config, &model.provider, &model.model, Some(port))?;
            }
        }
        Ok(config)
    }
}

fn add_listener(config: &mut RuntimeConfig, name: &str, port: u16) -> Result<(), String> {
    if port == 0 { return Err("Gateway port must be between 1 and 65535".into()); }
    config.listeners.push(Listener {
        name: name.into(), bind: SocketAddr::from(([0, 0, 0, 0], port)), protocol: ListenerProtocol::Http,
        virtual_hosts: vec![], tls_secret: None, security: ListenerSecurity::default(),
    });
    Ok(())
}

fn model_match(path: PathMatch, model: Option<String>) -> AgentRouteMatch {
    AgentRouteMatch { path, model, host: None, method: None, tool: None, agent: None, headers: vec![] }
}

fn model_backend(name: &str, provider: &str, model: &str) -> Backend {
    Backend { name: name.into(), kind: BackendKind::Llm {
        provider: provider.into(), models: vec![model.into()], endpoint: None,
        account_type: Some("api-key".into()), max_concurrency: None, credential_ref: None,
        quota_state: None, model_rewrites: BTreeMap::new(),
    }, policies: vec![] }
}

/// Used by both YAML compilation and Platform so adding a model also makes it routable.
pub fn add_model_binding(config: &mut RuntimeConfig, provider: &str, model: &str, port: Option<u16>) -> Result<(), String> {
    if config.provider(provider).is_none() { return Err(format!("Provider {provider} does not exist")); }
    if model.is_empty() || model.len() > 200 || model.chars().any(char::is_control) || (model.contains('*') && model != "*") {
        return Err("Model must be an exact ID or *".into());
    }
    let name = format!("{provider}/{model}");
    if config.backend(&name).is_some() { return Err(format!("Model binding {name} already exists")); }
    let port = port.or_else(|| config.listeners.iter().find(|l| l.protocol == ListenerProtocol::Http).map(|l| l.bind.port())).unwrap_or(4000);
    if !config.listeners.iter().any(|l| l.bind.port() == port && l.protocol == ListenerProtocol::Http) {
        add_listener(config, &format!("platform-{port}"), port)?;
    }
    config.backends.push(model_backend(&name, provider, model));
    let route_name = format!("platform-{port}/{model}");
    let destination = WeightedBackend { name, weight: 1, priority: None };
    if let Some(route) = config.routes.iter_mut().find(|route| route.name == route_name) {
        route.weighted_backends.push(destination);
    } else {
        let route = AgentRoute {
            name: route_name, protocol: AgentProtocol::Llm, listener_ports: vec![port],
            matches: vec![model_match(PathMatch::Prefix("/v1/".into()), (model != "*").then(|| model.into()))],
            weighted_backends: vec![destination], policies: vec![], replace_prefix_match: None,
        };
        let index = if model == "*" { config.routes.len() } else {
            config.routes.iter().position(|r| r.name == format!("platform-{port}/*")).unwrap_or(config.routes.len())
        };
        config.routes.insert(index, route);
    }
    Ok(())
}

pub fn configuration_schema() -> schemars::Schema {
    schemars::schema_for!(ConfigDocument)
}
