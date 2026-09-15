//! Per-backend stream observations. Missing usage never becomes a zero token rate.
use crate::llm::{LlmUsage, SseLines, UsageSink};
use axum::body::Body;
use hyper::body::{Bytes, HttpBody};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub(crate) struct Observation {
    started: Instant,
    usage: Mutex<Option<LlmUsage>>,
}

impl Observation {
    pub(crate) fn new(started: Instant) -> Arc<Self> {
        Arc::new(Self {
            started,
            usage: Mutex::new(None),
        })
    }
    pub(crate) fn sink(self: &Arc<Self>, sink: UsageSink) -> UsageSink {
        let observation = self.clone();
        Arc::new(move |usage| {
            *observation.usage.lock().unwrap() = Some(usage);
            sink(usage);
        })
    }
}

#[derive(Default)]
struct Counter {
    first_request: Option<Instant>,
    ttft_ms: f64,
    ttft_samples: u64,
    generation_seconds: f64,
    generated_tokens: u64,
    context_tokens: Option<u64>,
    requests: VecDeque<(u64, u64)>,
}

#[derive(Default)]
pub struct LlmTimings {
    counters: Mutex<BTreeMap<String, Counter>>,
}

#[derive(Default, Serialize)]
pub struct LlmTimingSummary {
    pub ttft_ms: Option<f64>,
    pub ttft_samples: u64,
    pub tokens_per_second: Option<f64>,
    pub generated_tokens: u64,
    pub generation_seconds: f64,
    pub requests_per_second: Option<f64>,
    pub context: Option<u64>,
}

impl LlmTimings {
    fn record(
        &self,
        backend: &str,
        observation: &Observation,
        first: Option<Instant>,
        complete: bool,
    ) {
        let now = Instant::now();
        let mut counters = self.counters.lock().unwrap();
        let counter = counters.entry(backend.to_string()).or_default();
        let origin = *counter.first_request.get_or_insert(observation.started);
        let second = now.duration_since(origin).as_secs();
        if let Some((_, count)) = counter.requests.back_mut().filter(|(t, _)| *t == second) {
            *count += 1;
        } else {
            counter.requests.push_back((second, 1));
        }
        while counter
            .requests
            .front()
            .is_some_and(|(t, _)| second.saturating_sub(*t) >= 60)
        {
            counter.requests.pop_front();
        }
        if let Some(first) = first {
            counter.ttft_ms += first.duration_since(observation.started).as_secs_f64() * 1000.0;
            counter.ttft_samples += 1;
            if complete {
                let duration = now.duration_since(first).as_secs_f64();
                if let Some(usage) = *observation.usage.lock().unwrap() {
                    if duration >= 0.001 && usage.completion_tokens > 0 {
                        counter.generation_seconds += duration;
                        counter.generated_tokens += usage.completion_tokens;
                    }
                }
            }
        }
        if let Some(usage) = *observation.usage.lock().unwrap() {
            counter.context_tokens = Some(counter.context_tokens.unwrap_or(0).max(usage.total()));
        }
    }

    pub fn summary(&self, backend: &str) -> LlmTimingSummary {
        let counters = self.counters.lock().unwrap();
        let Some(c) = counters.get(backend) else {
            return LlmTimingSummary::default();
        };
        let elapsed = c
            .first_request
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);
        let requests = c
            .requests
            .iter()
            .filter(|(t, _)| (elapsed as u64).saturating_sub(*t) < 60)
            .map(|(_, n)| n)
            .sum::<u64>();
        LlmTimingSummary {
            ttft_ms: (c.ttft_samples > 0).then(|| c.ttft_ms / c.ttft_samples as f64),
            ttft_samples: c.ttft_samples,
            tokens_per_second: (c.generation_seconds > 0.0)
                .then(|| c.generated_tokens as f64 / c.generation_seconds),
            generation_seconds: c.generation_seconds,
            generated_tokens: c.generated_tokens,
            requests_per_second: Some(requests as f64 / elapsed.clamp(1.0, 60.0)),
            context: c.context_tokens,
        }
    }
}

fn generated(value: &Value) -> bool {
    let nonempty = |v: &Value| v.as_str().is_some_and(|s| !s.is_empty());
    if value["choices"].as_array().is_some_and(|choices| {
        choices.iter().any(|c| {
            nonempty(&c["delta"]["content"])
                || nonempty(&c["delta"]["reasoning_content"])
                || c["delta"]["tool_calls"].as_array().is_some_and(|calls| {
                    calls
                        .iter()
                        .any(|call| nonempty(&call["function"]["arguments"]))
                })
        })
    }) {
        return true;
    }
    matches!(
        value["type"].as_str(),
        Some(
            "response.output_text.delta"
                | "response.function_call_arguments.delta"
                | "response.reasoning_summary_text.delta"
        )
    ) && nonempty(&value["delta"])
}

pub(crate) fn observe(
    body: Body,
    streaming: bool,
    backend: String,
    timings: Arc<LlmTimings>,
    observation: Arc<Observation>,
) -> Body {
    struct Stream {
        body: Body,
        lines: SseLines,
        streaming: bool,
        backend: String,
        timings: Arc<LlmTimings>,
        observation: Arc<Observation>,
        first: Option<Instant>,
        finished: bool,
        done: bool,
        bytes_without_content: usize,
    }
    impl Drop for Stream {
        fn drop(&mut self) {
            self.timings
                .record(&self.backend, &self.observation, self.first, self.finished);
        }
    }
    Body::wrap_stream(futures_util::stream::unfold(
        Stream {
            body,
            lines: SseLines::default(),
            streaming,
            backend,
            timings,
            observation,
            first: None,
            finished: false,
            done: false,
            bytes_without_content: 0,
        },
        |mut state| async move {
            if state.done {
                return None;
            }
            match state.body.data().await {
                Some(Ok(chunk)) => {
                    if state.streaming && state.first.is_none() {
                        state.bytes_without_content += chunk.len();
                        if state.bytes_without_content <= 1024 * 1024 {
                            for line in state.lines.push(&chunk) {
                                if serde_json::from_str::<Value>(&line)
                                    .is_ok_and(|value| generated(&value))
                                {
                                    state.first = Some(Instant::now());
                                    break;
                                }
                            }
                        }
                    }
                    Some((Ok::<Bytes, hyper::Error>(chunk), state))
                }
                Some(Err(error)) => {
                    state.done = true;
                    Some((Err(error), state))
                }
                None => {
                    state.finished = true;
                    drop(state);
                    None
                }
            }
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn completed_stream_records_generation_rate_and_context() {
        let timings = Arc::new(LlmTimings::default());
        let observation = Observation::new(Instant::now() - Duration::from_millis(20));
        *observation.usage.lock().unwrap() = Some(LlmUsage {
            prompt_tokens: 100,
            completion_tokens: 20,
            ..LlmUsage::default()
        });
        let upstream = Body::wrap_stream(futures_util::stream::unfold(0, |index| async move {
            match index {
                0 => Some((
                    Ok::<_, std::io::Error>(Bytes::from_static(
                        b"data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
                    )),
                    1,
                )),
                1 => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Some((Ok(Bytes::from_static(b"data: [DONE]\n\n")), 2))
                }
                _ => None,
            }
        }));
        let bytes = hyper::body::to_bytes(observe(
            upstream,
            true,
            "local".into(),
            timings.clone(),
            observation,
        ))
        .await
        .unwrap();
        assert!(bytes.ends_with(b"data: [DONE]\n\n"));
        let summary = timings.summary("local");
        assert!(summary.ttft_ms.unwrap() >= 20.0);
        assert_eq!(summary.ttft_samples, 1);
        assert!(summary.tokens_per_second.unwrap() > 0.0);
        assert_eq!(summary.generated_tokens, 20);
        assert_eq!(summary.context, Some(120));
        assert!(summary.requests_per_second.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn metadata_and_missing_usage_do_not_invent_token_measurements() {
        let timings = Arc::new(LlmTimings::default());
        let body = Body::from(
            "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\ndata: [DONE]\n\n",
        );
        hyper::body::to_bytes(observe(
            body,
            true,
            "local".into(),
            timings.clone(),
            Observation::new(Instant::now()),
        ))
        .await
        .unwrap();
        let summary = timings.summary("local");
        assert_eq!(summary.ttft_ms, None);
        assert_eq!(summary.tokens_per_second, None);
        assert_eq!(summary.context, None);
    }

    #[tokio::test]
    async fn cancelled_stream_does_not_count_completed_generation() {
        let timings = Arc::new(LlmTimings::default());
        let observation = Observation::new(Instant::now());
        *observation.usage.lock().unwrap() = Some(LlmUsage {
            completion_tokens: 20,
            ..LlmUsage::default()
        });
        let mut body = observe(
            Body::from("data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n"),
            true,
            "local".into(),
            timings.clone(),
            observation,
        );
        assert!(body.data().await.unwrap().is_ok());
        tokio::time::sleep(Duration::from_millis(2)).await;
        drop(body);
        assert_eq!(timings.summary("local").tokens_per_second, None);
    }
}
