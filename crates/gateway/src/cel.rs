use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use cel_interpreter::{Context, ExecutionError, Program, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct Expression {
    program: Arc<Program>,
    pub original_expression: String,
}

impl Expression {
    pub fn new(expression: &str) -> anyhow::Result<Self> {
        let program = Program::compile(expression)
            .map_err(|e| anyhow::anyhow!("failed to compile CEL expression '{}': {}", expression, e))?;
        Ok(Self {
            program: Arc::new(program),
            original_expression: expression.to_string(),
        })
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn eval(&self, context: &Context) -> Result<Value, ExecutionError> {
        self.program.execute(context)
    }

    pub fn eval_json(&self, value: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let mut ctx = Context::default();
        if let serde_json::Value::Object(map) = value {
            for (k, v) in map {
                ctx.add_variable(k.clone(), v.clone())
                    .map_err(|e| anyhow::anyhow!("failed to add CEL variable '{}': {:?}", k, e))?;
            }
        }
        let res = self.eval(&ctx)
            .map_err(|e| anyhow::anyhow!("failed to evaluate CEL expression '{}': {}", self.original_expression, e))?;
        let json_val = res.json()
            .map_err(|e| anyhow::anyhow!("failed to serialize CEL result to JSON: {:?}", e))?;
        Ok(json_val)
    }
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.original_expression == other.original_expression
    }
}

impl Eq for Expression {}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expression")
            .field("expression", &self.original_expression)
            .finish()
    }
}

impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.original_expression)
    }
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Expression::new(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Expression {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Expression".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({ "type": "string" })
    }
}

/// CEL Execution Context Schema representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ExecutorSerde {
    /// Incoming HTTP request attributes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestRefSerde>,

    /// HTTP response attributes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<ResponseRefSerde>,

    /// LLM attributes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmContextSerde>,

    /// Raw LLM request body before processing
    #[serde(rename = "llmRequest", default, skip_serializing_if = "Option::is_none")]
    pub llm_request: Option<serde_json::Value>,

    /// Claims from verified JWT
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jwt: Option<HashMap<String, serde_json::Value>>,

    /// Claims from verified API Key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<HashMap<String, serde_json::Value>>,

    /// Environment variables exposed to CEL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RequestRefSerde {
    pub method: String,
    pub uri: String,
    pub host: String,
    pub scheme: String,
    pub path: String,
    pub path_and_query: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ResponseRefSerde {
    pub code: u16,
    pub headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LlmContextSerde {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cel_expression_eval() {
        let expr = Expression::new("request.method == 'GET' && request.path.startsWith('/v1')").unwrap();
        let val = json!({
            "request": {
                "method": "GET",
                "path": "/v1/chat/completions"
            }
        });
        let res = expr.eval_json(&val).unwrap();
        assert_eq!(res, json!(true));
    }
}

