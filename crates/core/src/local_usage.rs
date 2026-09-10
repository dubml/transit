//! Local Claude Code and Codex CLI session logs.
//!
//! Reads JSONL the agents already write. No network, no cookies.

use crate::TokenCounts;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub const SOURCE_CLAUDE: &str = "local-claude";
pub const SOURCE_CODEX: &str = "local-codex";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalScanPaths {
    pub claude_roots: Vec<PathBuf>,
    pub codex_roots: Vec<PathBuf>,
}

impl LocalScanPaths {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let home = home_dir();
        let claude_roots = env_paths("TRANSIT_CLAUDE_HOME")
            .or_else(|| env_paths("CLAUDE_CONFIG_DIR"))
            .unwrap_or_else(|| {
                let mut roots = Vec::new();
                if let Some(home) = &home {
                    roots.push(home.join(".claude"));
                    roots.push(home.join(".config/claude"));
                }
                roots
            });
        let codex_roots = env_paths("TRANSIT_CODEX_HOME")
            .or_else(|| env_paths("CODEX_HOME"))
            .unwrap_or_else(|| {
                home.map(|home| vec![home.join(".codex")])
                    .unwrap_or_default()
            });
        Self {
            claude_roots,
            codex_roots,
        }
    }

    pub fn parse_roots(value: &str) -> Vec<PathBuf> {
        split_paths(value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LocalUsageRow {
    pub source: String,
    pub model: String,
    pub sessions: u64,
    pub requests: u64,
    pub prompt_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub cache_write_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LocalTick {
    pub day: String,
    pub model: String,
    pub t_ms: u64,
    pub tokens: u64,
    pub prompt_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub cache_write_tokens: u64,
    pub completion_tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LocalUsageReport {
    pub rows: Vec<LocalUsageRow>,
    pub ticks: Vec<LocalTick>,
    pub claude_files: u64,
    pub codex_files: u64,
    pub skipped_files: u64,
}

pub fn scan_local_usage(paths: &LocalScanPaths) -> LocalUsageReport {
    let mut report = LocalUsageReport::default();
    let mut acc = BTreeMap::<(String, String), Acc>::new();
    let mut days = BTreeMap::<(String, String), Acc>::new();
    let mut seen = HashSet::new();

    for root in unique_existing(&paths.claude_roots) {
        let dir = if root.join("projects").is_dir() {
            root.join("projects")
        } else {
            root
        };
        for file in jsonl_files(&dir) {
            match scan_claude_file(&file, &mut seen, &mut acc, &mut days) {
                Ok(true) => report.claude_files += 1,
                Ok(false) => {}
                Err(_) => report.skipped_files += 1,
            }
        }
    }
    for root in unique_existing(&paths.codex_roots) {
        let mut dirs = Vec::new();
        let sessions = root.join("sessions");
        let archived = root.join("archived_sessions");
        if sessions.is_dir() {
            dirs.push(sessions);
        }
        if archived.is_dir() {
            dirs.push(archived);
        }
        if dirs.is_empty() {
            dirs.push(root);
        }
        for dir in dirs {
            for file in jsonl_files(&dir) {
                match scan_codex_file(&file, &mut acc, &mut days) {
                    Ok(true) => report.codex_files += 1,
                    Ok(false) => {}
                    Err(_) => report.skipped_files += 1,
                }
            }
        }
    }

    report.rows = acc
        .into_iter()
        .map(|((source, model), row)| LocalUsageRow {
            source,
            model,
            sessions: row.sessions,
            requests: row.requests,
            prompt_tokens: row.prompt_tokens,
            cached_prompt_tokens: row.cached_prompt_tokens,
            cache_write_tokens: row.cache_write_tokens,
            completion_tokens: row.completion_tokens,
            total_tokens: row.prompt_tokens.saturating_add(row.completion_tokens),
        })
        .collect();
    report.ticks = days
        .into_iter()
        .filter_map(|((day, model), row)| {
            let t_ms = ymd_to_unix_ms(&day)?;
            Some(LocalTick {
                tokens: row.prompt_tokens.saturating_add(row.completion_tokens),
                prompt_tokens: row.prompt_tokens,
                cached_prompt_tokens: row.cached_prompt_tokens,
                cache_write_tokens: row.cache_write_tokens,
                completion_tokens: row.completion_tokens,
                day,
                model,
                t_ms,
            })
        })
        .collect();
    report
}

#[derive(Default)]
struct Acc {
    sessions: u64,
    requests: u64,
    prompt_tokens: u64,
    cached_prompt_tokens: u64,
    cache_write_tokens: u64,
    completion_tokens: u64,
}

impl Acc {
    fn add(&mut self, tokens: TokenCounts, requests: u64, sessions: u64) {
        self.sessions += sessions;
        self.requests += requests;
        self.prompt_tokens += tokens.prompt_tokens;
        self.cached_prompt_tokens += tokens.cached_prompt_tokens;
        self.cache_write_tokens += tokens.cache_write_tokens;
        self.completion_tokens += tokens.completion_tokens;
    }
}

fn merge(
    acc: &mut BTreeMap<(String, String), Acc>,
    source: &str,
    model: &str,
    tokens: TokenCounts,
    requests: u64,
) {
    if requests == 0 && tokens_zero(&tokens) {
        return;
    }
    let model = if model.is_empty() { "unknown" } else { model };
    acc.entry((source.to_string(), model.to_string()))
        .or_default()
        .add(tokens, requests, 1);
}

fn tokens_zero(tokens: &TokenCounts) -> bool {
    tokens.prompt_tokens == 0
        && tokens.cached_prompt_tokens == 0
        && tokens.cache_write_tokens == 0
        && tokens.completion_tokens == 0
}

fn add_day(
    days: &mut BTreeMap<(String, String), Acc>,
    day: Option<String>,
    model: &str,
    tokens: TokenCounts,
    requests: u64,
) {
    let Some(day) = day else {
        return;
    };
    let model = if model.is_empty() {
        "unknown".to_string()
    } else {
        model.to_string()
    };
    days.entry((day, model))
        .or_default()
        .add(tokens, requests, 0);
}

fn scan_claude_file(
    path: &Path,
    seen: &mut HashSet<String>,
    acc: &mut BTreeMap<(String, String), Acc>,
    days: &mut BTreeMap<(String, String), Acc>,
) -> std::io::Result<bool> {
    let file = File::open(path)?;
    let mut any = false;
    let mut per_model: BTreeMap<String, (TokenCounts, u64)> = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let uuid = value.get("uuid").and_then(Value::as_str).unwrap_or("");
        if uuid.is_empty() || !seen.insert(uuid.to_string()) {
            continue;
        }
        let message = match value.get("message") {
            Some(message) => message,
            None => continue,
        };
        let model = message.get("model").and_then(Value::as_str).unwrap_or("");
        if model.is_empty() || model == "<synthetic>" {
            continue;
        }
        let Some(usage) = message.get("usage") else {
            continue;
        };
        let tokens = claude_tokens(usage);
        if tokens_zero(&tokens) {
            continue;
        }
        let entry = per_model
            .entry(model.to_string())
            .or_insert((TokenCounts::default(), 0));
        add_counts(&mut entry.0, &tokens);
        entry.1 += 1;
        add_day(days, event_day(&value), model, tokens, 1);
        any = true;
    }
    for (model, (tokens, requests)) in per_model {
        merge(acc, SOURCE_CLAUDE, &model, tokens, requests);
    }
    Ok(any)
}

fn scan_codex_file(
    path: &Path,
    acc: &mut BTreeMap<(String, String), Acc>,
    days: &mut BTreeMap<(String, String), Acc>,
) -> std::io::Result<bool> {
    let file = File::open(path)?;
    let mut model = String::new();
    let mut has_last = false;
    let mut last_by_model: BTreeMap<String, (TokenCounts, u64)> = BTreeMap::new();
    let mut last_total: Option<(String, TokenCounts, Option<String>)> = None;
    let fallback_day = path_day(path);
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let ty = value.get("type").and_then(Value::as_str).unwrap_or("");
        let payload = value.get("payload");
        match ty {
            "world_state" => {
                if let Some(name) = payload
                    .and_then(|payload| payload.pointer("/state/model"))
                    .and_then(Value::as_str)
                {
                    if !name.is_empty() {
                        model = name.to_string();
                    }
                }
            }
            "session_meta" | "turn_context" => {
                if let Some(name) = payload
                    .and_then(|payload| payload.get("model"))
                    .and_then(Value::as_str)
                {
                    if !name.is_empty() {
                        model = name.to_string();
                    }
                }
            }
            "event_msg" => {
                let Some(payload) = payload else { continue };
                if payload.get("type").and_then(Value::as_str) != Some("token_count") {
                    continue;
                }
                let Some(info) = payload.get("info") else {
                    continue;
                };
                if let Some(last) = info.get("last_token_usage") {
                    has_last = true;
                    let tokens = openai_tokens(last);
                    if tokens_zero(&tokens) {
                        continue;
                    }
                    let key = if model.is_empty() {
                        "unknown".to_string()
                    } else {
                        model.clone()
                    };
                    let entry = last_by_model
                        .entry(key.clone())
                        .or_insert((TokenCounts::default(), 0));
                    add_counts(&mut entry.0, &tokens);
                    entry.1 += 1;
                    add_day(
                        days,
                        event_day(&value).or_else(|| fallback_day.clone()),
                        &key,
                        tokens,
                        1,
                    );
                } else if !has_last {
                    if let Some(total) = info.get("total_token_usage") {
                        last_total = Some((
                            if model.is_empty() {
                                "unknown".to_string()
                            } else {
                                model.clone()
                            },
                            openai_tokens(total),
                            event_day(&value).or_else(|| fallback_day.clone()),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    if has_last {
        if last_by_model.is_empty() {
            return Ok(false);
        }
        for (name, (tokens, requests)) in last_by_model {
            merge(acc, SOURCE_CODEX, &name, tokens, requests);
        }
        return Ok(true);
    }
    if let Some((name, tokens, day)) = last_total {
        if tokens_zero(&tokens) {
            return Ok(false);
        }
        add_day(days, day, &name, tokens, 1);
        merge(acc, SOURCE_CODEX, &name, tokens, 1);
        return Ok(true);
    }
    Ok(false)
}

fn claude_tokens(usage: &Value) -> TokenCounts {
    let uncached = json_u64(usage, "input_tokens");
    let cached = json_u64(usage, "cache_read_input_tokens");
    let write = json_u64(usage, "cache_creation_input_tokens");
    TokenCounts {
        prompt_tokens: uncached.saturating_add(cached),
        cached_prompt_tokens: cached,
        cache_write_tokens: write,
        completion_tokens: json_u64(usage, "output_tokens"),
    }
}

fn openai_tokens(usage: &Value) -> TokenCounts {
    let prompt = json_u64(usage, "input_tokens");
    let cached = json_u64(usage, "cached_input_tokens").min(prompt);
    TokenCounts {
        prompt_tokens: prompt,
        cached_prompt_tokens: cached,
        cache_write_tokens: json_u64(usage, "cache_write_input_tokens"),
        completion_tokens: json_u64(usage, "output_tokens"),
    }
}

fn add_counts(dst: &mut TokenCounts, src: &TokenCounts) {
    dst.prompt_tokens += src.prompt_tokens;
    dst.cached_prompt_tokens += src.cached_prompt_tokens;
    dst.cache_write_tokens += src.cache_write_tokens;
    dst.completion_tokens += src.completion_tokens;
}

fn json_u64(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn event_day(value: &Value) -> Option<String> {
    ymd(value.get("timestamp").and_then(Value::as_str)?)
}

fn path_day(path: &Path) -> Option<String> {
    let parts: Vec<&str> = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect();
    for window in parts.windows(3) {
        if let Some(day) = ymd(&format!("{}-{}-{}", window[0], window[1], window[2])) {
            return Some(day);
        }
    }
    None
}

fn ymd(value: &str) -> Option<String> {
    let day = value.get(0..10)?;
    let bytes = day.as_bytes();
    if bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                true
            } else {
                c.is_ascii_digit()
            }
        })
    {
        Some(day.to_string())
    } else {
        None
    }
}

fn ymd_to_unix_ms(day: &str) -> Option<u64> {
    let y: i32 = day.get(0..4)?.parse().ok()?;
    let m: u32 = day.get(5..7)?.parse().ok()?;
    let d: u32 = day.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = y as i64 - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 {
        i64::from(m) - 3
    } else {
        i64::from(m) + 9
    };
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    u64::try_from(days.checked_mul(86_400_000)?).ok()
}

fn jsonl_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_jsonl(root, &mut out);
    out.sort();
    out
}

fn walk_jsonl(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_jsonl(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
}

fn unique_existing(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for path in paths {
        if !path.exists() {
            continue;
        }
        let key = fs::canonicalize(path).unwrap_or_else(|_| path.clone());
        if seen.insert(key.clone()) {
            out.push(path.clone());
        }
    }
    out
}

fn env_paths(key: &str) -> Option<Vec<PathBuf>> {
    let value = std::env::var(key).ok()?;
    let paths = split_paths(&value);
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

fn split_paths(value: &str) -> Vec<PathBuf> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_jsonl(dir: &Path, name: &str, lines: &[&str]) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
        path
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "transit-local-usage-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            name
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn claude_dedupes_uuid_and_skips_synthetic() {
        let root = temp("claude");
        let projects = root.join("projects/demo");
        write_jsonl(
            &projects,
            "sess.jsonl",
            &[
                r#"{"type":"assistant","uuid":"a","timestamp":"2026-08-01T12:00:00Z","message":{"model":"claude-opus-5","usage":{"input_tokens":2,"cache_read_input_tokens":10,"cache_creation_input_tokens":4,"output_tokens":7}}}"#,
                r#"{"type":"assistant","uuid":"a","timestamp":"2026-08-01T12:00:00Z","message":{"model":"claude-opus-5","usage":{"input_tokens":2,"cache_read_input_tokens":10,"cache_creation_input_tokens":4,"output_tokens":7}}}"#,
                r#"{"type":"assistant","uuid":"b","timestamp":"2026-08-01T12:00:00Z","message":{"model":"<synthetic>","usage":{"input_tokens":9,"output_tokens":9}}}"#,
                r#"{"type":"assistant","uuid":"c","timestamp":"2026-08-02T12:00:00Z","message":{"model":"claude-opus-5","usage":{"input_tokens":3,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"output_tokens":8}}}"#,
            ],
        );
        let report = scan_local_usage(&LocalScanPaths {
            claude_roots: vec![root.clone()],
            codex_roots: vec![],
        });
        let _ = fs::remove_dir_all(&root);
        assert_eq!(report.rows.len(), 1);
        let row = &report.rows[0];
        assert_eq!(row.source, SOURCE_CLAUDE);
        assert_eq!(row.model, "claude-opus-5");
        assert_eq!(row.requests, 2);
        assert_eq!(row.prompt_tokens, 15);
        assert_eq!(row.cached_prompt_tokens, 10);
        assert_eq!(row.cache_write_tokens, 4);
        assert_eq!(row.completion_tokens, 15);
        assert_eq!(row.total_tokens, 30);
        assert_eq!(report.ticks.len(), 2);
        assert_eq!(report.ticks[0].day, "2026-08-01");
        assert_eq!(report.ticks[0].model, "claude-opus-5");
        assert_eq!(report.ticks[0].tokens, 19);
        assert_eq!(report.ticks[1].day, "2026-08-02");
        assert_eq!(report.ticks[1].tokens, 11);
    }

    #[test]
    fn codex_sums_last_token_usage() {
        let root = temp("codex");
        let sessions = root.join("sessions/2026/08/25");
        write_jsonl(
            &sessions,
            "rollout.jsonl",
            &[
                r#"{"type":"world_state","payload":{"state":{"model":"gpt-5.6-sol"}}}"#,
                r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"cache_write_input_tokens":5,"output_tokens":10},"total_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":10}}}}"#,
                r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":50,"cached_input_tokens":50,"output_tokens":3},"total_token_usage":{"input_tokens":150,"cached_input_tokens":90,"output_tokens":13}}}}"#,
            ],
        );
        let report = scan_local_usage(&LocalScanPaths {
            claude_roots: vec![],
            codex_roots: vec![root.clone()],
        });
        let _ = fs::remove_dir_all(&root);
        assert_eq!(report.rows.len(), 1);
        let row = &report.rows[0];
        assert_eq!(row.source, SOURCE_CODEX);
        assert_eq!(row.model, "gpt-5.6-sol");
        assert_eq!(row.requests, 2);
        assert_eq!(row.prompt_tokens, 150);
        assert_eq!(row.cached_prompt_tokens, 90);
        assert_eq!(row.cache_write_tokens, 5);
        assert_eq!(row.completion_tokens, 13);
        assert_eq!(row.total_tokens, 163);
        assert_eq!(report.ticks.len(), 1);
        assert_eq!(report.ticks[0].day, "2026-08-25");
        assert_eq!(report.ticks[0].model, "gpt-5.6-sol");
        assert_eq!(report.ticks[0].tokens, 163);
    }

    #[test]
    fn codex_reads_model_from_turn_context() {
        let root = temp("codex-turn");
        let sessions = root.join("sessions");
        write_jsonl(
            &sessions,
            "turn.jsonl",
            &[
                r#"{"type":"turn_context","payload":{"model":"gpt-5.4"}}"#,
                r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":8,"cached_input_tokens":2,"output_tokens":1}}}}"#,
            ],
        );
        let report = scan_local_usage(&LocalScanPaths {
            claude_roots: vec![],
            codex_roots: vec![root.clone()],
        });
        let _ = fs::remove_dir_all(&root);
        assert_eq!(report.rows[0].model, "gpt-5.4");
        assert_eq!(report.rows[0].prompt_tokens, 8);
    }

    #[test]
    fn codex_total_only_uses_last_snapshot() {
        let root = temp("codex-total");
        let sessions = root.join("sessions");
        write_jsonl(
            &sessions,
            "old.jsonl",
            &[
                r#"{"type":"session_meta","payload":{"model":"gpt-5.6-luna"}}"#,
                r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":1}}}}"#,
                r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":30,"cached_input_tokens":8,"output_tokens":4}}}}"#,
            ],
        );
        let report = scan_local_usage(&LocalScanPaths {
            claude_roots: vec![],
            codex_roots: vec![root.clone()],
        });
        let _ = fs::remove_dir_all(&root);
        let row = &report.rows[0];
        assert_eq!(row.model, "gpt-5.6-luna");
        assert_eq!(row.requests, 1);
        assert_eq!(row.prompt_tokens, 30);
        assert_eq!(row.cached_prompt_tokens, 8);
        assert_eq!(row.completion_tokens, 4);
    }
}
