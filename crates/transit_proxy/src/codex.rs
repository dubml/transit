//! ChatGPT OAuth uses the Codex Responses endpoint, including for chat clients.
use crate::llm::{self, LlmUsage, SseLines, Transcode, UsageSink};
use axum::body::Body;
use serde_json::{json, Value};
use std::collections::HashMap;

pub(crate) fn request(body: &Value, native: bool) -> Result<Value, String> {
    let mut out = if native {
        body.clone()
    } else {
        let messages = body
            .get("messages")
            .and_then(Value::as_array)
            .ok_or("Chat request requires messages")?;
        let mut input = Vec::new();
        let mut instructions = Vec::new();
        for message in messages {
            let role = message
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("user");
            if matches!(role, "system" | "developer") {
                match message.get("content") {
                    Some(Value::String(text)) => instructions.push(text.clone()),
                    Some(Value::Array(parts)) => {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                instructions.push(text.to_string());
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }
            if role == "tool" {
                input.push(json!({"type":"function_call_output","call_id":message["tool_call_id"],"output":message["content"]}));
                continue;
            }
            let mut content = Vec::new();
            match message.get("content") {
                Some(Value::String(text)) => content.push(json!({"type":if role=="assistant" {"output_text"} else {"input_text"},"text":text})),
                Some(Value::Array(parts)) => for part in parts {
                    match part["type"].as_str() {
                        Some("text") => content.push(json!({"type":if role=="assistant" {"output_text"} else {"input_text"},"text":part["text"]})),
                        Some("image_url") => content.push(json!({"type":"input_image","image_url":part["image_url"]["url"],"detail":part["image_url"].get("detail").cloned().unwrap_or(json!("auto"))})),
                        _ => return Err("Unsupported chat content for Codex OAuth".into()),
                    }
                },
                _ => {}
            }
            if !content.is_empty() {
                input.push(json!({"type":"message","role":role,"content":content}));
            }
            if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
                for call in calls {
                    input.push(json!({"type":"function_call","call_id":call["id"],"name":call["function"]["name"],"arguments":call["function"]["arguments"]}));
                }
            }
        }
        let mut out =
            json!({"model":body["model"],"input":input,"instructions":instructions.join("\n\n")});
        if let Some(tools) = body.get("tools").and_then(Value::as_array) {
            let mut translated = Vec::new();
            for tool in tools {
                if tool["type"] != "function" {
                    return Err("Unsupported chat tool for Codex OAuth".into());
                }
                let mut function = tool["function"].clone();
                if !function.is_object() || !function.get("name").is_some_and(Value::is_string) {
                    return Err("Chat function tools require a function object and name".into());
                }
                function["type"] = json!("function");
                translated.push(function);
            }
            out["tools"] = json!(translated);
        }
        if let Some(choice) = body.get("tool_choice") {
            out["tool_choice"] = if choice.is_string() {
                choice.clone()
            } else {
                json!({"type":"function","name":choice["function"]["name"]})
            };
        }
        if let Some(effort) = body.get("reasoning_effort") {
            out["reasoning"] = json!({"effort":effort});
        }
        if let Some(parallel) = body.get("parallel_tool_calls") {
            out["parallel_tool_calls"] = parallel.clone();
        }
        out
    };
    if !out.is_object() {
        return Err("Responses request must be an object".into());
    }
    out["stream"] = json!(true);
    out["store"] = json!(false);
    if out.get("instructions").is_none() {
        out["instructions"] = json!("");
    }
    for key in [
        "stream_options",
        "previous_response_id",
        "prompt_cache_retention",
        "safety_identifier",
        "max_output_tokens",
    ] {
        out.as_object_mut().unwrap().remove(key);
    }
    Ok(out)
}

pub(crate) fn usage(response: &Value) -> LlmUsage {
    let u = &response["usage"];
    let input = u["input_tokens"].as_u64().unwrap_or(0);
    let output = u["output_tokens"].as_u64().unwrap_or(0);
    LlmUsage {
        prompt_tokens: input,
        completion_tokens: output,
        cached_prompt_tokens: u["input_tokens_details"]["cached_tokens"]
            .as_u64()
            .unwrap_or(0)
            .min(input),
        reasoning_tokens: u["output_tokens_details"]["reasoning_tokens"]
            .as_u64()
            .unwrap_or(0)
            .min(output),
        cache_write_tokens: 0,
    }
}

pub(crate) fn chat_response(response: &Value) -> Value {
    let mut text = String::new();
    let mut calls = Vec::new();
    if let Some(items) = response["output"].as_array() {
        for item in items {
            if item["type"] == "function_call" {
                calls.push(json!({"id":item["call_id"],"type":"function","function":{"name":item["name"],"arguments":item["arguments"]}}));
            }
            if let Some(parts) = item["content"].as_array() {
                for part in parts {
                    if let Some(value) = part["text"].as_str() {
                        text.push_str(value);
                    }
                }
            }
        }
    }
    let mut message = json!({"role":"assistant","content":text});
    let finish = if calls.is_empty() {
        "stop"
    } else {
        message["tool_calls"] = json!(calls);
        "tool_calls"
    };
    let u = usage(response);
    json!({"id":response["id"],"object":"chat.completion","created":response["created_at"],"model":response["model"],
        "choices":[{"index":0,"message":message,"finish_reason":finish}],"usage":chat_usage(u)})
}

fn chat_usage(u: LlmUsage) -> Value {
    json!({"prompt_tokens":u.prompt_tokens,"completion_tokens":u.completion_tokens,"total_tokens":u.total(),
        "prompt_tokens_details":{"cached_tokens":u.cached_prompt_tokens},"completion_tokens_details":{"reasoning_tokens":u.reasoning_tokens}})
}

pub(crate) fn completed(bytes: &[u8]) -> Result<Value, String> {
    let mut lines = SseLines::default();
    let mut final_response = None;
    for line in lines.push(bytes) {
        if let Ok(event) = serde_json::from_str::<Value>(&line) {
            if event["type"] == "response.completed" {
                final_response = Some(event["response"].clone());
            }
            if matches!(
                event["type"].as_str(),
                Some("error" | "response.failed" | "response.incomplete")
            ) {
                return Err("Codex response did not complete".into());
            }
        }
    }
    final_response.ok_or_else(|| "Codex stream ended without a completed response".into())
}

struct CodexStream {
    lines: SseLines,
    sink: UsageSink,
    native: bool,
    include_usage: bool,
    id: String,
    model: String,
    completed: bool,
    tools: HashMap<u64, usize>,
}

impl CodexStream {
    fn chunk(&self, delta: Value, finish: Option<&str>) -> String {
        format!(
            "data: {}\n\n",
            json!({"id":self.id,"object":"chat.completion.chunk","model":self.model,
            "choices":[{"index":0,"delta":delta,"finish_reason":finish}]})
        )
    }
}

impl Transcode for CodexStream {
    fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        let mut out = String::new();
        for line in self.lines.push(chunk) {
            let Ok(event) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            match event["type"].as_str().unwrap_or("") {
                "response.created" => {
                    self.id = event["response"]["id"].as_str().unwrap_or("").to_string();
                }
                "response.output_text.delta" => {
                    out.push_str(&self.chunk(json!({"content":event["delta"]}), None))
                }
                "response.reasoning_summary_text.delta" => {
                    out.push_str(&self.chunk(json!({"reasoning_content":event["delta"]}), None))
                }
                "response.output_item.added" if event["item"]["type"] == "function_call" => {
                    let index = self.tools.len();
                    self.tools
                        .insert(event["output_index"].as_u64().unwrap_or(0), index);
                    out.push_str(&self.chunk(json!({"tool_calls":[{"index":index,"id":event["item"]["call_id"],"type":"function","function":{"name":event["item"]["name"],"arguments":""}}]}), None));
                }
                "response.function_call_arguments.delta" => {
                    if let Some(index) =
                        self.tools.get(&event["output_index"].as_u64().unwrap_or(0))
                    {
                        out.push_str(&self.chunk(json!({"tool_calls":[{"index":index,"function":{"arguments":event["delta"]}}]}), None));
                    }
                }
                "response.completed" if !self.completed => {
                    self.completed = true;
                    let u = usage(&event["response"]);
                    (self.sink)(u);
                    out.push_str(&self.chunk(
                        json!({}),
                        Some(if self.tools.is_empty() {
                            "stop"
                        } else {
                            "tool_calls"
                        }),
                    ));
                    if self.include_usage {
                        out.push_str(&format!("data: {}\n\n", json!({"id":self.id,"object":"chat.completion.chunk","model":self.model,"choices":[],"usage":chat_usage(u)})));
                    }
                    out.push_str("data: [DONE]\n\n");
                }
                "error" | "response.failed" | "response.incomplete" => {
                    self.completed = true;
                    out.push_str("data: {\"error\":{\"message\":\"Codex upstream response failed\",\"type\":\"upstream_error\"}}\n\n");
                }
                _ => {}
            }
        }
        if self.native {
            chunk.to_vec()
        } else {
            out.into_bytes()
        }
    }
    fn finish(&mut self) -> Vec<u8> {
        if self.completed {
            Vec::new()
        } else {
            b"data: {\"error\":{\"message\":\"Codex stream ended before completion\",\"type\":\"upstream_error\"}}\n\n".to_vec()
        }
    }
}

pub(crate) fn stream(
    body: Body,
    model: String,
    native: bool,
    include_usage: bool,
    sink: UsageSink,
) -> Body {
    llm::wrap_body(
        body,
        CodexStream {
            lines: SseLines::default(),
            sink,
            native,
            include_usage,
            id: String::new(),
            model,
            completed: false,
            tools: HashMap::new(),
        },
    )
}
