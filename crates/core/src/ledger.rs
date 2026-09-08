//! Offline token ledger.
//!
//! Converts observed token counts into published OpenAI API USD and ChatGPT
//! credits. No network I/O: unknown models fail instead of guessing.
//! ChatGPT credits and API dollars are separate published ledgers; this crate
//! does not invent a credits-to-USD FX rate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataQuality {
    Complete,
    Inconsistent,
    Unclassified,
}

impl Default for DataQuality {
    fn default() -> Self {
        Self::Complete
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingStatus {
    Exact,
    Unpriced,
    Missing,
    NoCatalog,
}

impl Default for PricingStatus {
    fn default() -> Self {
        Self::Unpriced
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionMode {
    Direct,
    Rollup,
}

impl Default for AttributionMode {
    fn default() -> Self {
        Self::Direct
    }
}

/// Mutually exclusive token buckets ensuring strict accounting integrity.
///
/// Invariants:
/// - input_uncached + cache_read == prompt_tokens
/// - output_non_reasoning + reasoning == completion_tokens
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBreakdown {
    pub input_uncached: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub output_non_reasoning: u64,
    pub reasoning: u64,
    pub unclassified: u64,
}

impl TokenBreakdown {
    pub fn new(
        prompt: u64,
        cache_read: u64,
        cache_write: u64,
        completion: u64,
        reasoning: u64,
    ) -> (Self, DataQuality) {
        let mut quality = DataQuality::Complete;
        let effective_cache_read = cache_read.min(prompt);
        let input_uncached = prompt.saturating_sub(effective_cache_read);
        if cache_read > prompt {
            quality = DataQuality::Inconsistent;
        }

        let effective_reasoning = reasoning.min(completion);
        let output_non_reasoning = completion.saturating_sub(effective_reasoning);
        if reasoning > completion {
            quality = DataQuality::Inconsistent;
        }

        (
            Self {
                input_uncached,
                cache_read: effective_cache_read,
                cache_write,
                output_non_reasoning,
                reasoning: effective_reasoning,
                unclassified: 0,
            },
            quality,
        )
    }

    pub fn total_input(&self) -> u64 {
        self.input_uncached + self.cache_read
    }

    pub fn total_output(&self) -> u64 {
        self.output_non_reasoning + self.reasoning
    }

    pub fn total(&self) -> u64 {
        self.total_input() + self.total_output() + self.unclassified
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostEvent {
    pub event_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub timestamp_ms: u64,
    pub route: String,
    pub backend: String,
    pub model: String,
    pub provider: String,
    pub account: String,
    pub protocol: String,
    pub operation: String,
    pub token_breakdown: TokenBreakdown,
    pub latency_ms: u64,
    pub ttft_ms: Option<u64>,
    pub status_code: u16,
    pub data_quality: DataQuality,
    pub pricing_status: PricingStatus,
    pub api_usd_nanos: Option<UsdNanos>,
    pub api_usd: Option<String>,
    pub chatgpt_credit_micros: Option<CreditMicros>,
    pub chatgpt_credits: Option<String>,
    pub attribution_mode: AttributionMode,
    pub io_bytes: u64,
    pub retries: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendModelRow {
    pub model: String,
    pub provider: String,
    pub requests: u64,
    pub uncached_input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub api_usd: String,
    pub chatgpt_credits: String,
    pub pricing_status: PricingStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendAccountRow {
    pub account: String,
    pub provider: String,
    pub requests: u64,
    pub api_usd: String,
    pub chatgpt_credits: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendLedgerSummary {
    pub total_api_usd_nanos: UsdNanos,
    pub total_api_usd: String,
    pub total_chatgpt_credit_micros: CreditMicros,
    pub total_chatgpt_credits: String,
    pub priced_requests: u64,
    pub unpriced_requests: u64,
    pub models: Vec<SpendModelRow>,
    pub accounts: Vec<SpendAccountRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheTierBreakdown {
    pub provider_cache_read_tokens: u64,
    pub provider_cache_write_tokens: u64,
    pub gateway_prefix_cache_tokens: u64,
    pub agent_context_tokens_reduced: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenLedgerSummary {
    pub total_input_uncached: u64,
    pub total_cache_read: u64,
    pub total_cache_write: u64,
    pub total_output_non_reasoning: u64,
    pub total_reasoning: u64,
    pub total_unclassified: u64,
    pub total_tokens: u64,
    pub cache_hit_rate_pct: f64,
    pub context_saved_tokens: u64,
    pub complete_events: u64,
    pub inconsistent_events: u64,
    pub tiers: CacheTierBreakdown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpEfficiencyRow {
    pub tool: String,
    pub backend: String,
    pub calls: u64,
    pub failures: u64,
    pub io_bytes: u64,
    pub avg_latency_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct A2aEfficiencyRow {
    pub method: String,
    pub backend: String,
    pub calls: u64,
    pub failures: u64,
    pub io_bytes: u64,
    pub avg_latency_ms: u64,
    pub direct_calls: u64,
    pub rollup_calls: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EfficiencyLedgerSummary {
    pub mcp_calls: u64,
    pub mcp_failures: u64,
    pub mcp_io_bytes: u64,
    pub a2a_calls: u64,
    pub a2a_failures: u64,
    pub a2a_io_bytes: u64,
    pub total_retries: u64,
    pub retry_overhead_tokens: u64,
    pub mcp_tools: Vec<McpEfficiencyRow>,
    pub a2a_methods: Vec<A2aEfficiencyRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub category: String,
    pub description: String,
    pub evidence: String,
    pub potential_tokens_saved: u64,
    pub potential_usd_saved: Option<String>,
    pub realized: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationLedgerSummary {
    pub tokens_saved_cache: u64,
    pub tokens_saved_rtk: u64,
    pub tokens_saved_condense: u64,
    pub gross_potential_tokens: u64,
    pub opportunities: Vec<OptimizationOpportunity>,
}

pub const API_USD_SOURCE: &str = "https://developers.openai.com/api/docs/pricing";
pub const ANTHROPIC_USD_SOURCE: &str = "https://platform.claude.com/docs/en/about-claude/pricing";
pub const CHATGPT_CREDITS_SOURCE: &str = "https://learn.chatgpt.com/docs/pricing";
pub const CHATGPT_FAST_SOURCE: &str = "https://learn.chatgpt.com/docs/agent-configuration/speed";
pub const RATE_CARD_AS_OF: &str = "2026-08-28";
/// GPT-5.6 ChatGPT Fast credits are 2.5x Standard. API Fast USD is 2x Standard.
pub const CHATGPT_FAST_CREDIT_NUMER: u128 = 5;
pub const CHATGPT_FAST_CREDIT_DENOM: u128 = 2;
pub const TOKENS_PER_MILLION: u128 = 1_000_000;

/// $1 = 1_000_000_000 nanodollars.
pub type UsdNanos = u128;
/// 1 ChatGPT credit = 1_000_000 microcredits.
pub type CreditMicros = u128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Standard,
    Fast,
    Flex,
    Batch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextBand {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct TokenCounts {
    pub prompt_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub cache_write_tokens: u64,
    pub completion_tokens: u64,
}

impl TokenCounts {
    pub fn uncached_prompt_tokens(&self) -> u64 {
        self.prompt_tokens.saturating_sub(self.cached_prompt_tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    UnknownModel {
        model: String,
    },
    CachedExceedsPrompt {
        prompt_tokens: u64,
        cached_prompt_tokens: u64,
    },
    MissingApiUsdRate {
        model: String,
        component: &'static str,
    },
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownModel { model } => {
                write!(f, "no published rate card row for model {model}")
            }
            Self::CachedExceedsPrompt {
                prompt_tokens,
                cached_prompt_tokens,
            } => write!(
                f,
                "cached_prompt_tokens {cached_prompt_tokens} exceeds prompt_tokens {prompt_tokens}"
            ),
            Self::MissingApiUsdRate { model, component } => {
                write!(
                    f,
                    "model {model} has no published API USD rate for {component}"
                )
            }
        }
    }
}

impl std::error::Error for LedgerError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LineItem {
    pub component: &'static str,
    pub tokens: u64,
    pub api_usd_nanos: UsdNanos,
    pub chatgpt_credit_micros: Option<CreditMicros>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Quote {
    pub model: String,
    pub tier: ServiceTier,
    pub context: ContextBand,
    pub tokens: TokenCounts,
    pub uncached_prompt_tokens: u64,
    pub line_items: Vec<LineItem>,
    pub api_usd_nanos: UsdNanos,
    pub chatgpt_credit_micros: CreditMicros,
    pub chatgpt_credits_complete: bool,
    pub api_usd_source: &'static str,
    pub chatgpt_credits_source: &'static str,
    pub chatgpt_fast_source: &'static str,
    pub rate_card_as_of: &'static str,
    pub formula: &'static str,
    pub vendor: &'static str,
}

impl Quote {
    pub fn api_usd(&self) -> String {
        format_usd_nanos(self.api_usd_nanos)
    }

    pub fn chatgpt_credits(&self) -> String {
        format_credit_micros(self.chatgpt_credit_micros)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RateCardEntry {
    pub model: &'static str,
    pub vendor: &'static str,
    pub tier: ServiceTier,
    pub context: ContextBand,
    pub input_usd_per_1m: String,
    pub cached_usd_per_1m: Option<String>,
    pub output_usd_per_1m: String,
    pub input_credits_per_1m: Option<String>,
    pub cached_credits_per_1m: Option<String>,
    pub output_credits_per_1m: Option<String>,
}

pub fn published_rate_card() -> Vec<RateCardEntry> {
    RATE_CARD
        .iter()
        .map(|row| RateCardEntry {
            model: row.model,
            vendor: usd_vendor(row.model),
            tier: row.tier,
            context: row.context,
            input_usd_per_1m: format_usd_nanos(row.input_usd_nanos_per_m),
            cached_usd_per_1m: row.cached_usd_nanos_per_m.map(format_usd_nanos),
            output_usd_per_1m: format_usd_nanos(row.output_usd_nanos_per_m),
            input_credits_per_1m: row.input_credit_micros_per_m.map(format_credit_micros),
            cached_credits_per_1m: row.cached_credit_micros_per_m.map(format_credit_micros),
            output_credits_per_1m: row.output_credit_micros_per_m.map(format_credit_micros),
        })
        .collect()
}

#[derive(Clone, Copy)]
struct RateRow {
    model: &'static str,
    tier: ServiceTier,
    context: ContextBand,
    input_usd_nanos_per_m: UsdNanos,
    cached_usd_nanos_per_m: Option<UsdNanos>,
    cache_write_usd_nanos_per_m: Option<UsdNanos>,
    output_usd_nanos_per_m: UsdNanos,
    input_credit_micros_per_m: Option<CreditMicros>,
    cached_credit_micros_per_m: Option<CreditMicros>,
    output_credit_micros_per_m: Option<CreditMicros>,
}

const fn usd_per_m(dollars: u64, micros: u32) -> UsdNanos {
    // dollars.micros USD per 1M tokens, as nanodollars.
    (dollars as u128) * 1_000_000_000 + (micros as u128) * 1_000
}

const fn credits_per_m(credits: u64, micros: u32) -> CreditMicros {
    (credits as u128) * 1_000_000 + (micros as u128)
}

const fn chatgpt_fast_credits(standard: CreditMicros) -> CreditMicros {
    standard * CHATGPT_FAST_CREDIT_NUMER / CHATGPT_FAST_CREDIT_DENOM
}

fn mul_per_million(tokens: u64, per_million: u128) -> u128 {
    (tokens as u128) * per_million / TOKENS_PER_MILLION
}

pub fn normalize_model(model: &str) -> String {
    model.trim().to_ascii_lowercase().replace('_', "-")
}

pub fn quote(
    model: &str,
    tokens: TokenCounts,
    tier: ServiceTier,
    context: ContextBand,
) -> Result<Quote, LedgerError> {
    if tokens.cached_prompt_tokens > tokens.prompt_tokens {
        return Err(LedgerError::CachedExceedsPrompt {
            prompt_tokens: tokens.prompt_tokens,
            cached_prompt_tokens: tokens.cached_prompt_tokens,
        });
    }
    let model = normalize_model(model);
    let aliased = model.replace('.', "-");
    let row = RATE_CARD
        .iter()
        .copied()
        .find(|row| {
            (row.model == model || row.model == aliased)
                && row.tier == tier
                && row.context == context
        })
        .ok_or_else(|| LedgerError::UnknownModel {
            model: model.clone(),
        })?;

    let uncached = tokens.uncached_prompt_tokens();
    let mut items = Vec::new();
    let mut usd = 0u128;
    let mut credits = 0u128;
    let mut credits_complete = true;

    push_item(
        &mut items,
        &mut usd,
        &mut credits,
        &mut credits_complete,
        "uncached_input",
        uncached,
        Some(row.input_usd_nanos_per_m),
        row.input_credit_micros_per_m,
        &model,
    )?;
    if tokens.cached_prompt_tokens > 0 {
        push_item(
            &mut items,
            &mut usd,
            &mut credits,
            &mut credits_complete,
            "cached_input",
            tokens.cached_prompt_tokens,
            row.cached_usd_nanos_per_m,
            row.cached_credit_micros_per_m,
            &model,
        )?;
    }
    if tokens.cache_write_tokens > 0 {
        push_item(
            &mut items,
            &mut usd,
            &mut credits,
            &mut credits_complete,
            "cache_write",
            tokens.cache_write_tokens,
            row.cache_write_usd_nanos_per_m,
            None,
            &model,
        )?;
    }
    push_item(
        &mut items,
        &mut usd,
        &mut credits,
        &mut credits_complete,
        "output",
        tokens.completion_tokens,
        Some(row.output_usd_nanos_per_m),
        row.output_credit_micros_per_m,
        &model,
    )?;

    let vendor = usd_vendor(&model);
    Ok(Quote {
        model,
        tier,
        context,
        tokens,
        uncached_prompt_tokens: uncached,
        line_items: items,
        api_usd_nanos: usd,
        chatgpt_credit_micros: credits,
        chatgpt_credits_complete: credits_complete,
        api_usd_source: if vendor == "anthropic" {
            ANTHROPIC_USD_SOURCE
        } else {
            API_USD_SOURCE
        },
        chatgpt_credits_source: CHATGPT_CREDITS_SOURCE,
        chatgpt_fast_source: CHATGPT_FAST_SOURCE,
        rate_card_as_of: RATE_CARD_AS_OF,
        formula: "api_usd = tokens * api_rate_per_1M / 1e6; subscription_credits = tokens * chatgpt_credit_rate_per_1M / 1e6; GPT-5.6 Fast credits = 2.5x Standard credits; uncached_input = prompt_tokens - cached_prompt_tokens",
        vendor,
    })
}

fn usd_vendor(model: &str) -> &'static str {
    if model.starts_with("claude") {
        "anthropic"
    } else {
        "openai"
    }
}

fn push_item(
    items: &mut Vec<LineItem>,
    usd_total: &mut u128,
    credits_total: &mut u128,
    credits_complete: &mut bool,
    component: &'static str,
    tokens: u64,
    usd_per_m: Option<UsdNanos>,
    credit_per_m: Option<CreditMicros>,
    model: &str,
) -> Result<(), LedgerError> {
    let Some(usd_per_m) = usd_per_m else {
        return Err(LedgerError::MissingApiUsdRate {
            model: model.to_string(),
            component,
        });
    };
    let api_usd_nanos = mul_per_million(tokens, usd_per_m);
    *usd_total += api_usd_nanos;
    let chatgpt_credit_micros = if let Some(rate) = credit_per_m {
        let add = mul_per_million(tokens, rate);
        *credits_total += add;
        Some(add)
    } else {
        if tokens > 0 {
            *credits_complete = false;
        }
        None
    };
    items.push(LineItem {
        component,
        tokens,
        api_usd_nanos,
        chatgpt_credit_micros,
    });
    Ok(())
}

pub fn format_usd_nanos(nanos: UsdNanos) -> String {
    let dollars = nanos / 1_000_000_000;
    let frac = nanos % 1_000_000_000;
    let mut s = format!("{dollars}.{frac:09}");
    while s.ends_with('0') && s.contains('.') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    format!("${s}")
}

pub fn format_credit_micros(micros: CreditMicros) -> String {
    let whole = micros / 1_000_000;
    let frac = micros % 1_000_000;
    let mut s = format!("{whole}.{frac:06}");
    while s.ends_with('0') && s.contains('.') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    s
}

// Current Codex models only: GPT-5.6 Sol / Terra / Luna.
// API USD: developers.openai.com/api/docs/pricing, 2026-08-22.
// ChatGPT credits: learn.chatgpt.com/docs/pricing (Standard) and
// learn.chatgpt.com/docs/agent-configuration/speed (GPT-5.6 Fast = 2.5x credits).
// Flex/Batch have no Codex credit table; same-token Standard credits are used
// as the subscription side of the dual quote.
const SOL_IN: CreditMicros = credits_per_m(100, 0);
const SOL_CACHE: CreditMicros = credits_per_m(10, 0);
const SOL_OUT: CreditMicros = credits_per_m(500, 0);
const TERRA_IN: CreditMicros = credits_per_m(50, 0);
const TERRA_CACHE: CreditMicros = credits_per_m(5, 0);
const TERRA_OUT: CreditMicros = credits_per_m(300, 0);
const LUNA_IN: CreditMicros = credits_per_m(5, 0);
const LUNA_CACHE: CreditMicros = credits_per_m(0, 500_000);
const LUNA_OUT: CreditMicros = credits_per_m(30, 0);

const RATE_CARD: &[RateRow] = &[
    RateRow {
        model: "gpt-5.6-sol",
        tier: ServiceTier::Standard,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(4, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 400_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(5, 0)),
        output_usd_nanos_per_m: usd_per_m(20, 0),
        input_credit_micros_per_m: Some(SOL_IN),
        cached_credit_micros_per_m: Some(SOL_CACHE),
        output_credit_micros_per_m: Some(SOL_OUT),
    },
    RateRow {
        model: "gpt-5.6-sol",
        tier: ServiceTier::Standard,
        context: ContextBand::Long,
        input_usd_nanos_per_m: usd_per_m(8, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 800_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(10, 0)),
        output_usd_nanos_per_m: usd_per_m(30, 0),
        input_credit_micros_per_m: Some(SOL_IN),
        cached_credit_micros_per_m: Some(SOL_CACHE),
        output_credit_micros_per_m: Some(SOL_OUT),
    },
    RateRow {
        model: "gpt-5.6-sol",
        tier: ServiceTier::Fast,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(8, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 800_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(10, 0)),
        output_usd_nanos_per_m: usd_per_m(40, 0),
        input_credit_micros_per_m: Some(chatgpt_fast_credits(SOL_IN)),
        cached_credit_micros_per_m: Some(chatgpt_fast_credits(SOL_CACHE)),
        output_credit_micros_per_m: Some(chatgpt_fast_credits(SOL_OUT)),
    },
    RateRow {
        model: "gpt-5.6-sol",
        tier: ServiceTier::Flex,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(2, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 200_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(2, 500_000)),
        output_usd_nanos_per_m: usd_per_m(10, 0),
        input_credit_micros_per_m: Some(SOL_IN),
        cached_credit_micros_per_m: Some(SOL_CACHE),
        output_credit_micros_per_m: Some(SOL_OUT),
    },
    RateRow {
        model: "gpt-5.6-sol",
        tier: ServiceTier::Batch,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(2, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 200_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(2, 500_000)),
        output_usd_nanos_per_m: usd_per_m(10, 0),
        input_credit_micros_per_m: Some(SOL_IN),
        cached_credit_micros_per_m: Some(SOL_CACHE),
        output_credit_micros_per_m: Some(SOL_OUT),
    },
    RateRow {
        model: "gpt-5.6-terra",
        tier: ServiceTier::Standard,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(2, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 200_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(2, 500_000)),
        output_usd_nanos_per_m: usd_per_m(12, 0),
        input_credit_micros_per_m: Some(TERRA_IN),
        cached_credit_micros_per_m: Some(TERRA_CACHE),
        output_credit_micros_per_m: Some(TERRA_OUT),
    },
    RateRow {
        model: "gpt-5.6-terra",
        tier: ServiceTier::Standard,
        context: ContextBand::Long,
        input_usd_nanos_per_m: usd_per_m(4, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 400_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(5, 0)),
        output_usd_nanos_per_m: usd_per_m(18, 0),
        input_credit_micros_per_m: Some(TERRA_IN),
        cached_credit_micros_per_m: Some(TERRA_CACHE),
        output_credit_micros_per_m: Some(TERRA_OUT),
    },
    RateRow {
        model: "gpt-5.6-terra",
        tier: ServiceTier::Fast,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(4, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 400_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(5, 0)),
        output_usd_nanos_per_m: usd_per_m(24, 0),
        input_credit_micros_per_m: Some(chatgpt_fast_credits(TERRA_IN)),
        cached_credit_micros_per_m: Some(chatgpt_fast_credits(TERRA_CACHE)),
        output_credit_micros_per_m: Some(chatgpt_fast_credits(TERRA_OUT)),
    },
    RateRow {
        model: "gpt-5.6-terra",
        tier: ServiceTier::Flex,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(1, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 100_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(1, 250_000)),
        output_usd_nanos_per_m: usd_per_m(6, 0),
        input_credit_micros_per_m: Some(TERRA_IN),
        cached_credit_micros_per_m: Some(TERRA_CACHE),
        output_credit_micros_per_m: Some(TERRA_OUT),
    },
    RateRow {
        model: "gpt-5.6-terra",
        tier: ServiceTier::Batch,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(1, 0),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 100_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(1, 250_000)),
        output_usd_nanos_per_m: usd_per_m(6, 0),
        input_credit_micros_per_m: Some(TERRA_IN),
        cached_credit_micros_per_m: Some(TERRA_CACHE),
        output_credit_micros_per_m: Some(TERRA_OUT),
    },
    RateRow {
        model: "gpt-5.6-luna",
        tier: ServiceTier::Standard,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(0, 200_000),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 20_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(0, 250_000)),
        output_usd_nanos_per_m: usd_per_m(1, 200_000),
        input_credit_micros_per_m: Some(LUNA_IN),
        cached_credit_micros_per_m: Some(LUNA_CACHE),
        output_credit_micros_per_m: Some(LUNA_OUT),
    },
    RateRow {
        model: "gpt-5.6-luna",
        tier: ServiceTier::Standard,
        context: ContextBand::Long,
        input_usd_nanos_per_m: usd_per_m(0, 400_000),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 40_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(0, 500_000)),
        output_usd_nanos_per_m: usd_per_m(1, 800_000),
        input_credit_micros_per_m: Some(LUNA_IN),
        cached_credit_micros_per_m: Some(LUNA_CACHE),
        output_credit_micros_per_m: Some(LUNA_OUT),
    },
    RateRow {
        model: "gpt-5.6-luna",
        tier: ServiceTier::Fast,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(0, 400_000),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 40_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(0, 500_000)),
        output_usd_nanos_per_m: usd_per_m(2, 400_000),
        input_credit_micros_per_m: Some(chatgpt_fast_credits(LUNA_IN)),
        cached_credit_micros_per_m: Some(chatgpt_fast_credits(LUNA_CACHE)),
        output_credit_micros_per_m: Some(chatgpt_fast_credits(LUNA_OUT)),
    },
    RateRow {
        model: "gpt-5.6-luna",
        tier: ServiceTier::Flex,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(0, 100_000),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 10_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(0, 125_000)),
        output_usd_nanos_per_m: usd_per_m(0, 600_000),
        input_credit_micros_per_m: Some(LUNA_IN),
        cached_credit_micros_per_m: Some(LUNA_CACHE),
        output_credit_micros_per_m: Some(LUNA_OUT),
    },
    RateRow {
        model: "gpt-5.6-luna",
        tier: ServiceTier::Batch,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(0, 100_000),
        cached_usd_nanos_per_m: Some(usd_per_m(0, 10_000)),
        cache_write_usd_nanos_per_m: Some(usd_per_m(0, 125_000)),
        output_usd_nanos_per_m: usd_per_m(0, 600_000),
        input_credit_micros_per_m: Some(LUNA_IN),
        cached_credit_micros_per_m: Some(LUNA_CACHE),
        output_credit_micros_per_m: Some(LUNA_OUT),
    },
    std_row("gpt-5.5", 5, 0, Some(usd_per_m(0, 500_000)), None, 30, 0),
    std_row("gpt-5.5", 10, 0, Some(usd_per_m(1, 0)), None, 45, 0).long(),
    std_row("gpt-5.5-pro", 30, 0, None, None, 180, 0),
    std_row("gpt-5.5-pro", 60, 0, None, None, 270, 0).long(),
    std_row(
        "gpt-5.4",
        2,
        500_000,
        Some(usd_per_m(0, 250_000)),
        None,
        15,
        0,
    ),
    std_row(
        "gpt-5.4",
        5,
        0,
        Some(usd_per_m(0, 500_000)),
        None,
        22,
        500_000,
    )
    .long(),
    std_row(
        "gpt-5.4-mini",
        0,
        750_000,
        Some(usd_per_m(0, 75_000)),
        None,
        4,
        500_000,
    ),
    std_row(
        "gpt-5.4-nano",
        0,
        200_000,
        Some(usd_per_m(0, 20_000)),
        None,
        1,
        250_000,
    ),
    std_row("gpt-5.4-pro", 30, 0, None, None, 180, 0),
    std_row("gpt-5.4-pro", 60, 0, None, None, 270, 0).long(),
    std_row(
        "gpt-5.2",
        1,
        750_000,
        Some(usd_per_m(0, 175_000)),
        None,
        14,
        0,
    ),
    std_row("gpt-5.2-pro", 21, 0, None, None, 168, 0),
    std_row(
        "gpt-5.1",
        1,
        250_000,
        Some(usd_per_m(0, 125_000)),
        None,
        10,
        0,
    ),
    std_row(
        "gpt-5",
        1,
        250_000,
        Some(usd_per_m(0, 125_000)),
        None,
        10,
        0,
    ),
    std_row(
        "gpt-5-mini",
        0,
        250_000,
        Some(usd_per_m(0, 25_000)),
        None,
        2,
        0,
    ),
    std_row(
        "gpt-5-nano",
        0,
        50_000,
        Some(usd_per_m(0, 5_000)),
        None,
        0,
        400_000,
    ),
    std_row("gpt-5-pro", 15, 0, None, None, 120, 0),
    std_row(
        "claude-fable-5",
        10,
        0,
        Some(usd_per_m(1, 0)),
        Some(usd_per_m(12, 500_000)),
        50,
        0,
    ),
    std_row(
        "claude-mythos-5",
        10,
        0,
        Some(usd_per_m(1, 0)),
        Some(usd_per_m(12, 500_000)),
        50,
        0,
    ),
    std_row(
        "claude-opus-5",
        5,
        0,
        Some(usd_per_m(0, 500_000)),
        Some(usd_per_m(6, 250_000)),
        25,
        0,
    ),
    std_row(
        "claude-opus-4-8",
        5,
        0,
        Some(usd_per_m(0, 500_000)),
        Some(usd_per_m(6, 250_000)),
        25,
        0,
    ),
    std_row(
        "claude-opus-4-7",
        5,
        0,
        Some(usd_per_m(0, 500_000)),
        Some(usd_per_m(6, 250_000)),
        25,
        0,
    ),
    std_row(
        "claude-opus-4-6",
        5,
        0,
        Some(usd_per_m(0, 500_000)),
        Some(usd_per_m(6, 250_000)),
        25,
        0,
    ),
    std_row(
        "claude-opus-4-5",
        5,
        0,
        Some(usd_per_m(0, 500_000)),
        Some(usd_per_m(6, 250_000)),
        25,
        0,
    ),
    std_row(
        "claude-sonnet-5",
        2,
        0,
        Some(usd_per_m(0, 200_000)),
        Some(usd_per_m(2, 500_000)),
        10,
        0,
    ),
    std_row(
        "claude-sonnet-4-6",
        3,
        0,
        Some(usd_per_m(0, 300_000)),
        Some(usd_per_m(3, 750_000)),
        15,
        0,
    ),
    std_row(
        "claude-sonnet-4-5",
        3,
        0,
        Some(usd_per_m(0, 300_000)),
        Some(usd_per_m(3, 750_000)),
        15,
        0,
    ),
    std_row(
        "claude-haiku-4-5",
        1,
        0,
        Some(usd_per_m(0, 100_000)),
        Some(usd_per_m(1, 250_000)),
        5,
        0,
    ),
    std_row(
        "claude-opus-5",
        10,
        0,
        Some(usd_per_m(1, 0)),
        Some(usd_per_m(12, 500_000)),
        50,
        0,
    )
    .fast(),
    std_row(
        "claude-opus-4-8",
        10,
        0,
        Some(usd_per_m(1, 0)),
        Some(usd_per_m(12, 500_000)),
        50,
        0,
    )
    .fast(),
];

const fn std_row(
    model: &'static str,
    in_dollars: u64,
    in_micros: u32,
    cached: Option<UsdNanos>,
    write: Option<UsdNanos>,
    out_dollars: u64,
    out_micros: u32,
) -> RateRow {
    RateRow {
        model,
        tier: ServiceTier::Standard,
        context: ContextBand::Short,
        input_usd_nanos_per_m: usd_per_m(in_dollars, in_micros),
        cached_usd_nanos_per_m: cached,
        cache_write_usd_nanos_per_m: write,
        output_usd_nanos_per_m: usd_per_m(out_dollars, out_micros),
        input_credit_micros_per_m: None,
        cached_credit_micros_per_m: None,
        output_credit_micros_per_m: None,
    }
}

impl RateRow {
    const fn long(mut self) -> Self {
        self.context = ContextBand::Long;
        self
    }

    const fn fast(mut self) -> Self {
        self.tier = ServiceTier::Fast;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(prompt: u64, cached: u64, completion: u64) -> TokenCounts {
        TokenCounts {
            prompt_tokens: prompt,
            cached_prompt_tokens: cached,
            cache_write_tokens: 0,
            completion_tokens: completion,
        }
    }

    #[test]
    fn million_sol_input_is_four_dollars() {
        let q = quote(
            "gpt-5.6-sol",
            counts(1_000_000, 0, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(q.api_usd_nanos, 4_000_000_000);
        assert_eq!(q.api_usd(), "$4.0");
        assert_eq!(q.chatgpt_credit_micros, 100_000_000);
        assert_eq!(q.chatgpt_credits(), "100.0");
        assert!(q.chatgpt_credits_complete);
    }

    #[test]
    fn splits_cached_prompt_from_uncached() {
        let q = quote(
            "gpt-5.6-sol",
            counts(1_000_000, 250_000, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(q.uncached_prompt_tokens, 750_000);
        // 0.75M * $4 + 0.25M * $0.40 = $3.00 + $0.10 = $3.10
        assert_eq!(q.api_usd_nanos, 3_100_000_000);
        // 0.75M * 100 + 0.25M * 10 = 75 + 2.5 = 77.5 credits
        assert_eq!(q.chatgpt_credit_micros, 77_500_000);
        assert!(q.chatgpt_credits_complete);
    }

    #[test]
    fn one_token_sol_input_is_4000_nanos() {
        let q = quote(
            "GPT-5.6-Sol",
            counts(1, 0, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(q.api_usd_nanos, 4_000);
        assert_eq!(q.api_usd(), "$0.000004");
    }

    #[test]
    fn luna_cached_thousand_tokens_half_milli_credit() {
        let q = quote(
            "gpt-5.6-luna",
            counts(1_000, 1_000, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(q.chatgpt_credit_micros, 500);
        assert_eq!(q.chatgpt_credits(), "0.0005");
        // $0.02 / 1M * 1000 = $0.00002
        assert_eq!(q.api_usd_nanos, 20_000);
        assert!(q.chatgpt_credits_complete);
    }

    #[test]
    fn fast_sol_doubles_standard_usd() {
        let std = quote(
            "gpt-5.6-sol",
            counts(1_000_000, 0, 1_000_000),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        let fast = quote(
            "gpt-5.6-sol",
            counts(1_000_000, 0, 1_000_000),
            ServiceTier::Fast,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(std.api_usd_nanos, 24_000_000_000);
        assert_eq!(fast.api_usd_nanos, 48_000_000_000);
        // API Fast USD is 2x Standard. ChatGPT Fast credits are 2.5x Standard.
        assert_eq!(std.chatgpt_credit_micros, 600_000_000);
        assert_eq!(fast.chatgpt_credit_micros, 1_500_000_000);
        assert!(fast.chatgpt_credits_complete);
    }

    #[test]
    fn unknown_model_fails() {
        let err = quote(
            "not-a-model",
            counts(1, 0, 1),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap_err();
        assert!(matches!(err, LedgerError::UnknownModel { .. }));
    }

    #[test]
    fn cached_gt_prompt_fails() {
        let err = quote(
            "gpt-5.6-sol",
            counts(10, 11, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap_err();
        assert!(matches!(err, LedgerError::CachedExceedsPrompt { .. }));
    }

    #[test]
    fn cache_write_keeps_known_credits_and_flags_gap() {
        let mut tokens = counts(1_000_000, 0, 0);
        tokens.cache_write_tokens = 1_000_000;
        let q = quote(
            "gpt-5.6-sol",
            tokens,
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        // $4 input + $5 cache write
        assert_eq!(q.api_usd_nanos, 9_000_000_000);
        // input credits counted; cache write has no published ChatGPT credit row
        assert_eq!(q.chatgpt_credit_micros, 100_000_000);
        assert!(!q.chatgpt_credits_complete);
    }

    #[test]
    fn rate_card_keeps_gpt56_and_adds_published_models() {
        let card = published_rate_card();
        assert!(card.iter().any(|row| row.model == "gpt-5.6-sol"
            && row.tier == ServiceTier::Standard
            && row.context == ContextBand::Short
            && row.input_usd_per_1m == "$4.0"
            && row.input_credits_per_1m.as_deref() == Some("100.0")));
        assert!(card.iter().any(|row| row.model == "gpt-5.5"
            && row.vendor == "openai"
            && row.input_usd_per_1m == "$5.0"));
        assert!(card.iter().any(|row| row.model == "claude-opus-5"
            && row.vendor == "anthropic"
            && row.input_usd_per_1m == "$5.0"
            && row.output_usd_per_1m == "$25.0"));
        assert!(card
            .iter()
            .any(|row| row.model == "claude-fable-5" && row.input_usd_per_1m == "$10.0"));
    }

    #[test]
    fn unpublished_models_fail_closed() {
        for model in ["gpt-5.3-codex", "Gemma4-26B-A4B", "Qwen3-8B-Uncensored"] {
            let err = quote(
                model,
                counts(1, 0, 1),
                ServiceTier::Standard,
                ContextBand::Short,
            )
            .unwrap_err();
            assert!(matches!(err, LedgerError::UnknownModel { .. }), "{model}");
        }
    }

    #[test]
    fn gpt55_and_claude_opus_quote_official_usd() {
        let gpt = quote(
            "gpt-5.5",
            counts(1_000_000, 0, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(gpt.api_usd_nanos, 5_000_000_000);
        assert!(!gpt.chatgpt_credits_complete);
        let claude = quote(
            "claude-opus-4.8",
            counts(1_000_000, 0, 0),
            ServiceTier::Standard,
            ContextBand::Short,
        )
        .unwrap();
        assert_eq!(claude.model, "claude-opus-4.8");
        assert_eq!(claude.api_usd_nanos, 5_000_000_000);
        assert_eq!(claude.vendor, "anthropic");
    }

    #[test]
    fn token_breakdown_enforces_mutual_exclusivity_and_quality() {
        let (tb, quality) = TokenBreakdown::new(1000, 300, 50, 500, 100);
        assert_eq!(quality, DataQuality::Complete);
        assert_eq!(tb.input_uncached, 700);
        assert_eq!(tb.cache_read, 300);
        assert_eq!(tb.cache_write, 50);
        assert_eq!(tb.output_non_reasoning, 400);
        assert_eq!(tb.reasoning, 100);
        assert_eq!(tb.total_input(), 1000);
        assert_eq!(tb.total_output(), 500);
        assert_eq!(tb.total(), 1500);

        // Inconsistent test: cache_read exceeds prompt
        let (tb_bad, quality_bad) = TokenBreakdown::new(100, 150, 0, 50, 60);
        assert_eq!(quality_bad, DataQuality::Inconsistent);
        assert_eq!(tb_bad.cache_read, 100);
        assert_eq!(tb_bad.input_uncached, 0);
        assert_eq!(tb_bad.reasoning, 50);
        assert_eq!(tb_bad.output_non_reasoning, 0);
    }
}

