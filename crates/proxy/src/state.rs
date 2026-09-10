use crate::activation::Activator;
use transit_core::{
    format_credit_micros, format_usd_nanos, A2aEfficiencyRow, ApplyOutcome, AttributionMode,
    CacheTierBreakdown, Cluster, ConfigConflict, ConfigDelta, ConfigSnapshot, ConfigStore,
    CostEvent, DataQuality, EfficiencyLedgerSummary, Endpoint, McpEfficiencyRow,
    OptimizationLedgerSummary, OptimizationOpportunity, OutlierDetectionConfig, PricingStatus,
    RateLimitPolicy, Result, RuntimeConfig, SecretKeyReference, SecurityDecision, SourceId,
    SourceState, SpendAccountRow, SpendLedgerSummary, SpendModelRow, TokenBreakdown, TokenCounts,
    TokenLedgerSummary, TokenLimitPolicy, WeightedBackend, WeightedCluster, TransitError,
};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const COST_TICK_CAP: usize = 512;
const COST_EVENT_CAP: usize = 1024;
const SECURITY_DECISION_CAP: usize = 512;

const LATENCY_BUCKETS_MS: [u64; 7] = [5, 10, 25, 50, 100, 250, 1000];

/// Concurrency tracker for one scope: the gateway as a whole, or one route.
///
/// It reports two things because neither alone is enough to scale on. `current`
/// is what a scrape sees at that instant, which misses everything between
/// scrapes. `micros_total` is the integral of `current` over time, so
/// `rate(..._seconds_total[1m])` yields the true average concurrency for that
/// minute no matter when Prometheus happened to poll.
///
/// Concurrency is tracked rather than derived from the latency histogram
/// because that histogram only records requests that finished. A gateway
/// stalled on slow upstreams — exactly the case worth scaling for — reports
/// nothing there until the backlog drains.
#[derive(Debug)]
struct Concurrency {
    current: u64,
    micros_total: u128,
    updated: Instant,
}

impl Default for Concurrency {
    fn default() -> Self {
        Self {
            current: 0,
            micros_total: 0,
            updated: Instant::now(),
        }
    }
}

impl Concurrency {
    /// Folds the time spent at the present level into the integral. Called
    /// before every change and before every read, so the integral is exact
    /// rather than sampled.
    fn accumulate(&mut self, now: Instant) {
        self.micros_total += self.current as u128 * now.duration_since(self.updated).as_micros();
        self.updated = now;
    }

    fn enter(&mut self, now: Instant) {
        self.accumulate(now);
        self.current += 1;
    }

    fn exit(&mut self, now: Instant) {
        self.accumulate(now);
        // Saturating because a guard must never be able to wrap the gauge into
        // a nonsense value, whatever order drops happen in.
        self.current = self.current.saturating_sub(1);
    }

    fn snapshot(&mut self, now: Instant) -> ConcurrencyMetric {
        self.accumulate(now);
        ConcurrencyMetric {
            in_flight: self.current,
            seconds_total: self.micros_total as f64 / 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ConcurrencyMetric {
    /// Requests in flight at the moment of the read.
    pub in_flight: u64,
    /// Accumulated request time. `rate()` over it gives average concurrency.
    pub seconds_total: f64,
}

/// Decrements the gateway-wide concurrency when the request ends. A request can
/// leave through a policy denial, an upstream error or a dropped connection, so
/// the decrement rides on `Drop` rather than on any single exit path.
pub struct RequestGuard {
    state: ProxyState,
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        let mut metrics = self.state.inner.metrics.lock().unwrap();
        metrics.in_flight.exit(Instant::now());
    }
}

/// Same contract as [`RequestGuard`], scoped to one route and cluster.
pub struct RouteGuard {
    state: ProxyState,
    key: String,
}

impl Drop for RouteGuard {
    fn drop(&mut self) {
        let mut metrics = self.state.inner.metrics.lock().unwrap();
        if let Some(route) = metrics.http_route_in_flight.get_mut(&self.key) {
            route.concurrency.exit(Instant::now());
        }
    }
}

#[derive(Clone)]
pub struct ProxyState {
    inner: Arc<Inner>,
}

struct Inner {
    llm_accounts: crate::LlmAccounts,
    access_settings: crate::access_settings::AccessSettings,
    llm_timings: Arc<crate::LlmTimings>,
    /// Shared with every configuration source. Sources write deltas into it
    /// directly; the proxy only ever reads published snapshots.
    store: Arc<ConfigStore>,
    /// Store revision whose stale hot-state keys have already been pruned.
    /// Pruning is driven from the read path rather than the write path because
    /// sources write to the store without going through `ProxyState`.
    pruned_revision: AtomicU64,
    /// Backs the [`ProxyState::apply_config`] helper: one state-of-the-world
    /// tracker per owning source.
    document_sources: Mutex<BTreeMap<SourceId, SourceState>>,
    // Round-robin cursors, one per selection domain: routes rotate over their
    // weighted clusters/backends, clusters rotate over their endpoints. A single
    // shared counter made each domain's sequence depend on unrelated traffic, so
    // whenever domains had different sizes the distribution skewed away from the
    // configured weights.
    route_pickers: Mutex<HashMap<String, u64>>,
    endpoint_pickers: Mutex<HashMap<String, u64>>,
    rate_limits: Mutex<HashMap<String, RateLimitBucket>>,
    token_usage: Mutex<HashMap<String, TokenBucket>>,
    circuit_breakers: Mutex<HashMap<String, CircuitBreakerBucket>>,
    outliers: Mutex<HashMap<String, OutlierBucket>>,
    mcp_sessions: Mutex<BindingMap>,
    a2a_tasks: Mutex<BindingMap>,
    credentials: RwLock<HashMap<SecretKeyReference, String>>,
    security_decisions: Mutex<VecDeque<SecurityDecision>>,
    metrics: Mutex<MetricsStore>,
    /// Holds requests for scaled-to-zero targets. Lives here rather than on
    /// the server because endpoint selection — the only thing that can tell a
    /// cold target from a dead one — lives here too.
    activation: Activator,
}

#[derive(Debug, Clone, Serialize)]
pub struct Readiness {
    pub ready: bool,
    /// Monotonic store revision, bumped once per applied delta.
    pub revision: u64,
    /// `source=version` labels joined by commas.
    pub version: String,
    /// Version label reported by each source that has published.
    pub source_versions: BTreeMap<SourceId, String>,
    pub conflicts: Vec<ConfigConflict>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProxyMetrics {
    /// Requests parked waiting for a scaled-to-zero target to come up.
    /// Separated from in-flight requests so a cold start is not read as the
    /// gateway being slow.
    pub held_activation_requests: u64,
    pub total_requests: u64,
    pub agent_requests: u64,
    pub policy_denied: u64,
    pub upstream_failures: u64,
    pub concurrency: ConcurrencyMetric,
    pub http_route_concurrency: Vec<HttpRouteConcurrencyMetric>,
    pub http_routes: Vec<HttpRouteMetric>,
    pub routes: Vec<RouteMetric>,
    pub llm_usage: Vec<LlmUsageMetric>,
    pub mcp_tools: Vec<McpToolMetric>,
    pub a2a_methods: Vec<A2aMethodMetric>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LlmUsageMetric {
    pub route: String,
    pub backend: String,
    pub model: String,
    pub requests: u64,
    pub prompt_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub priced_requests: u64,
    pub estimated_usd_nanos: u128,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct McpToolMetric {
    pub route: String,
    pub backend: String,
    pub tool: String,
    pub calls: u64,
    pub failures: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct A2aMethodMetric {
    pub route: String,
    pub backend: String,
    pub method: String,
    pub calls: u64,
    pub failures: u64,
}

// Affinity bindings (MCP session -> backend, A2A task -> backend) are bounded
// so abandoned keys cannot grow proxy memory without limit; last_used
// refreshes on every routed request.
const BINDING_TTL: Duration = Duration::from_secs(60 * 60);
const BINDING_CAP: usize = 10_000;

#[derive(Debug)]
struct Binding {
    backend: String,
    last_used: Instant,
}

#[derive(Debug, Default)]
struct BindingMap {
    entries: HashMap<String, Binding>,
}

impl BindingMap {
    fn bind(&mut self, key: String, backend: String) {
        // The O(n) sweeps only run once the map is actually full.
        if self.entries.len() >= BINDING_CAP {
            self.entries
                .retain(|_, binding| binding.last_used.elapsed() < BINDING_TTL);
        }
        if self.entries.len() >= BINDING_CAP {
            // Bindings are an affinity optimization; dropping the idlest one
            // only costs that key its stickiness, not correctness.
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, binding)| binding.last_used)
                .map(|(key, _)| key.clone())
            {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(
            key,
            Binding {
                backend,
                last_used: Instant::now(),
            },
        );
    }

    fn lookup(&mut self, key: &str) -> Option<String> {
        let binding = self.entries.get_mut(key)?;
        if binding.last_used.elapsed() >= BINDING_TTL {
            self.entries.remove(key);
            return None;
        }
        binding.last_used = Instant::now();
        Some(binding.backend.clone())
    }

    fn remove(&mut self, key: &str) {
        self.entries.remove(key);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpRouteMetric {
    pub namespace: String,
    pub gateway: String,
    pub route: String,
    pub cluster: String,
    pub method: String,
    pub status_code: u16,
    pub requests: u64,
    pub failures: u64,
    pub latency_ms_sum: u64,
    pub latency_ms_buckets: Vec<LatencyBucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpRouteConcurrencyMetric {
    pub namespace: String,
    pub gateway: String,
    pub route: String,
    pub cluster: String,
    pub concurrency: ConcurrencyMetric,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteMetric {
    pub protocol: String,
    pub route: String,
    pub backend: String,
    pub requests: u64,
    pub failures: u64,
    pub latency_ms_sum: u64,
    pub latency_ms_buckets: Vec<LatencyBucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyBucket {
    pub le: u64,
    pub count: u64,
}

#[derive(Debug, Default)]
struct MetricsStore {
    total_requests: u64,
    agent_requests: u64,
    policy_denied: u64,
    upstream_failures: u64,
    in_flight: Concurrency,
    http_routes: HashMap<String, HttpRouteMetricCounter>,
    http_route_in_flight: HashMap<String, HttpRouteConcurrencyCounter>,
    routes: HashMap<String, RouteMetricCounter>,
    llm_usage: HashMap<String, LlmUsageMetric>,
    cost_ticks: VecDeque<CostTick>,
    cost_events: VecDeque<CostEvent>,
    mcp_tools: HashMap<String, McpToolMetric>,
    a2a_methods: HashMap<String, A2aMethodMetric>,
}

/// One quoted LLM call. Cost Control tape / flame chart plots these.
#[derive(Debug, Clone, Serialize)]
pub struct CostTick {
    pub t_ms: u64,
    pub model: String,
    pub usd: f64,
    pub credits: f64,
}

/// Per-route concurrency is keyed without method or status code: both are
/// unknown while the request is still in flight, which is the only time this
/// counter means anything.
#[derive(Debug, Default)]
struct HttpRouteConcurrencyCounter {
    namespace: String,
    gateway: String,
    route: String,
    cluster: String,
    concurrency: Concurrency,
}

#[derive(Debug, Default)]
struct HttpRouteMetricCounter {
    namespace: String,
    gateway: String,
    route: String,
    cluster: String,
    method: String,
    status_code: u16,
    requests: u64,
    failures: u64,
    latency_ms_sum: u64,
    latency_ms_buckets: [u64; LATENCY_BUCKETS_MS.len()],
}

#[derive(Debug, Default)]
struct RouteMetricCounter {
    protocol: String,
    route: String,
    backend: String,
    requests: u64,
    failures: u64,
    latency_ms_sum: u64,
    latency_ms_buckets: [u64; LATENCY_BUCKETS_MS.len()],
}

#[derive(Debug)]
struct RateLimitBucket {
    window_started: Instant,
    used: u32,
}

#[derive(Debug)]
struct TokenBucket {
    window_started: Instant,
    window: Duration,
    used: u64,
}

impl TokenBucket {
    fn roll_window(&mut self) {
        if self.window_started.elapsed() >= self.window {
            self.window_started = Instant::now();
            self.used = 0;
        }
    }
}

#[derive(Debug, Default)]
struct CircuitBreakerBucket {
    active: u32,
}

// Envoy's defaults, applied when the control plane leaves a field unset.
const DEFAULT_CONSECUTIVE_5XX: u32 = 5;
const DEFAULT_BASE_EJECTION_TIME: Duration = Duration::from_secs(30);
const DEFAULT_MAX_EJECTION_PERCENT: u32 = 10;

#[derive(Debug, Default)]
struct OutlierBucket {
    consecutive_failures: u32,
    // Grows the ejection window for repeat offenders, as Envoy does.
    ejections: u32,
    ejected_until: Option<Instant>,
}

fn outlier_key(cluster: &str, endpoint: &Endpoint) -> String {
    format!("{cluster}|{}:{}", endpoint.address, endpoint.port)
}

/// How many of `total` endpoints may be ejected at once, per `max_ejection_percent`
/// and `min_health_percent`. Ejecting a whole cluster into unavailability would turn
/// a partial outage into a total one, so the caps are honored even when every
/// endpoint is failing.
fn ejection_allowance(total: usize, cfg: &OutlierDetectionConfig) -> usize {
    let max_pct = cfg
        .max_ejection_percent
        .unwrap_or(DEFAULT_MAX_EJECTION_PERCENT)
        .min(100) as usize;
    let by_max = total * max_pct / 100;
    let by_min_health = match cfg.min_health_percent {
        Some(pct) => total.saturating_sub((total * pct.min(100) as usize).div_ceil(100)),
        None => total,
    };
    by_max.min(by_min_health)
}

/// Parses the duration forms the xDS client emits (`"30s"`, `"0.500000000s"`) and the
/// `"500ms"` form static YAML tends to use. Unparseable values fall back to `default`.
fn parse_duration(raw: Option<&String>, default: Duration) -> Duration {
    let Some(raw) = raw.map(|value| value.trim()) else {
        return default;
    };
    let seconds = if let Some(ms) = raw.strip_suffix("ms") {
        ms.parse::<f64>().ok().map(|value| value / 1000.0)
    } else {
        raw.strip_suffix('s').and_then(|s| s.parse::<f64>().ok())
    };
    // from_secs_f64 panics on negative or non-finite input.
    seconds
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(Duration::from_secs_f64)
        .unwrap_or(default)
}

/// Advances `key`'s round-robin cursor and returns its position within `modulus`.
/// Only the first sighting of a key allocates; the key set is bounded by config.
fn next_cursor(pickers: &Mutex<HashMap<String, u64>>, key: &str, modulus: u64) -> u64 {
    if modulus == 0 {
        return 0;
    }
    let mut pickers = pickers.lock().unwrap();
    let cursor = match pickers.get_mut(key) {
        Some(cursor) => cursor,
        None => pickers.entry(key.to_string()).or_insert(0),
    };
    let current = *cursor;
    *cursor = cursor.wrapping_add(1);
    current % modulus
}

pub struct CircuitBreakerPermit {
    state: ProxyState,
    cluster: String,
}

impl Drop for CircuitBreakerPermit {
    fn drop(&mut self) {
        self.state.release_circuit_breaker(&self.cluster);
    }
}

impl Default for ProxyState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyState {
    /// A proxy over a fresh, empty store. Configuration arrives when a source
    /// applies its first delta.
    pub fn new() -> Self {
        Self::with_store(Arc::new(ConfigStore::new()))
    }

    /// A proxy over a store shared with the configuration sources.
    pub fn with_store(store: Arc<ConfigStore>) -> Self {
        Self::with_activator(store, Activator::from_env())
    }

    /// A proxy whose activator is supplied rather than read from the
    /// environment, so a test can exercise activation without setting process
    /// state that every other test in the binary would then share.
    pub fn with_activator(store: Arc<ConfigStore>, activation: Activator) -> Self {
        Self {
            inner: Arc::new(Inner {
                llm_accounts: crate::LlmAccounts::default(),
                access_settings: crate::access_settings::AccessSettings::default(),
                llm_timings: Arc::new(crate::LlmTimings::default()),
                store,
                pruned_revision: AtomicU64::new(0),
                document_sources: Mutex::new(BTreeMap::new()),
                route_pickers: Mutex::new(HashMap::new()),
                endpoint_pickers: Mutex::new(HashMap::new()),
                rate_limits: Mutex::new(HashMap::new()),
                token_usage: Mutex::new(HashMap::new()),
                circuit_breakers: Mutex::new(HashMap::new()),
                outliers: Mutex::new(HashMap::new()),
                mcp_sessions: Mutex::new(BindingMap::default()),
                a2a_tasks: Mutex::new(BindingMap::default()),
                credentials: RwLock::new(HashMap::new()),
                security_decisions: Mutex::new(VecDeque::new()),
                metrics: Mutex::new(MetricsStore::default()),
                activation,
            }),
        }
    }

    /// The store every configuration source writes into.
    pub fn store(&self) -> &Arc<ConfigStore> {
        &self.inner.store
    }

    pub fn llm_accounts(&self) -> &crate::LlmAccounts {
        &self.inner.llm_accounts
    }

    pub fn access_settings(&self) -> &crate::access_settings::AccessSettings {
        &self.inner.access_settings
    }

    pub fn llm_timings(&self) -> &Arc<crate::LlmTimings> {
        &self.inner.llm_timings
    }

    /// The current published configuration. One atomic refcount bump: nothing
    /// in the configuration is copied, so the request path can hold it for the
    /// whole request.
    pub fn snapshot(&self) -> Arc<ConfigSnapshot> {
        let snapshot = self.inner.store.snapshot();
        self.prune_stale_state(&snapshot);
        snapshot
    }

    pub fn replace_credentials(&self, values: HashMap<SecretKeyReference, String>) {
        *self.inner.credentials.write().unwrap() = values;
    }

    pub fn credential(&self, reference: &SecretKeyReference) -> Option<String> {
        self.inner
            .credentials
            .read()
            .unwrap()
            .get(reference)
            .cloned()
    }

    /// Applies a delta on behalf of `source`.
    pub fn apply_delta(&self, source: SourceId, delta: ConfigDelta) -> ApplyOutcome {
        let outcome = self.inner.store.apply(source, delta);
        self.prune_stale_state(&self.inner.store.snapshot());
        outcome
    }

    /// Publishes a whole control-plane document for tests and single-shot
    /// bootstrapping. Production updates use the same xDS owner and apply deltas
    /// directly to the store.
    pub fn apply_config(&self, cfg: RuntimeConfig) -> std::result::Result<(), Vec<ConfigConflict>> {
        self.apply_config_from(SourceId::Xds, cfg)
    }

    pub fn apply_config_from(
        &self,
        source: SourceId,
        cfg: RuntimeConfig,
    ) -> std::result::Result<(), Vec<ConfigConflict>> {
        let delta = {
            let mut sources = self.inner.document_sources.lock().unwrap();
            sources.entry(source).or_default().reconcile(cfg)
        };
        let outcome = self.apply_delta(source, delta);
        let mut problems = outcome.rejected;
        problems.extend(outcome.conflicts);
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }

    pub fn readiness(&self) -> Readiness {
        let snapshot = self.inner.store.snapshot();
        Readiness {
            ready: snapshot.ready(),
            revision: snapshot.revision(),
            version: snapshot.version().to_string(),
            source_versions: snapshot.source_versions().clone(),
            conflicts: snapshot.conflicts().to_vec(),
        }
    }

    /// Drops hot-state keyed on resources the current configuration no longer
    /// contains: round-robin cursors, circuit-breaker counters, and outlier
    /// buckets would otherwise linger for the process lifetime.
    ///
    /// Runs at most once per store revision. Sources write to the store without
    /// going through `ProxyState`, so this is driven from the read path; the
    /// revision check makes the steady-state cost one relaxed atomic load.
    fn prune_stale_state(&self, snapshot: &ConfigSnapshot) {
        let revision = snapshot.revision();
        let pruned = self.inner.pruned_revision.load(Ordering::Acquire);
        if pruned == revision
            || self
                .inner
                .pruned_revision
                .compare_exchange(pruned, revision, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return;
        }

        let cluster_names: std::collections::HashSet<&str> = snapshot
            .clusters()
            .map(|cluster| cluster.name.as_str())
            .collect();
        self.inner
            .circuit_breakers
            .lock()
            .unwrap()
            .retain(|name, bucket| cluster_names.contains(name.as_str()) || bucket.active > 0);
        self.inner
            .endpoint_pickers
            .lock()
            .unwrap()
            .retain(|name, _| cluster_names.contains(name.as_str()));
        // Outlier keys are "{cluster}|{addr}:{port}"; drop those whose cluster
        // is gone, and those whose endpoint no longer appears in it.
        let endpoint_keys: std::collections::HashSet<String> = snapshot
            .clusters()
            .flat_map(|cluster| {
                cluster
                    .endpoints
                    .iter()
                    .map(|endpoint| outlier_key(&cluster.name, endpoint))
            })
            .collect();
        self.inner
            .outliers
            .lock()
            .unwrap()
            .retain(|key, _| endpoint_keys.contains(key));
        let route_names: std::collections::HashSet<&str> = snapshot.route_names().collect();
        self.inner
            .route_pickers
            .lock()
            .unwrap()
            .retain(|name, _| route_names.contains(name.as_str()));
    }

    pub async fn pick_cluster<'a>(
        &self,
        route: &str,
        clusters: &'a [WeightedCluster],
    ) -> Option<&'a WeightedCluster> {
        let total: u32 = clusters.iter().map(|c| c.weight).sum();
        if total == 0 {
            return clusters.first();
        }
        let next = next_cursor(&self.inner.route_pickers, route, u64::from(total)) as u32;
        let mut cursor = 0;
        clusters.iter().find(|cluster| {
            cursor += cluster.weight;
            next < cursor
        })
    }

    pub async fn pick_backend<'a>(
        &self,
        route: &str,
        backends: &'a [WeightedBackend],
    ) -> Option<&'a WeightedBackend> {
        let has_priority = backends.iter().any(|b| b.priority.is_some());
        if has_priority {
            let min_priority = backends
                .iter()
                .filter_map(|b| b.priority)
                .min()
                .unwrap_or(1);
            let tier_backends: Vec<&'a WeightedBackend> = backends
                .iter()
                .filter(|b| b.priority.unwrap_or(u32::MAX) == min_priority)
                .collect();
            let total: u32 = tier_backends.iter().map(|b| b.weight).sum();
            if total == 0 {
                return tier_backends.first().copied();
            }
            let next = next_cursor(&self.inner.route_pickers, route, u64::from(total)) as u32;
            let mut cursor = 0;
            return tier_backends.into_iter().find(|backend| {
                cursor += backend.weight;
                next < cursor
            });
        }

        let total: u32 = backends.iter().map(|b| b.weight).sum();
        if total == 0 {
            return backends.first();
        }
        let next = next_cursor(&self.inner.route_pickers, route, u64::from(total)) as u32;
        let mut cursor = 0;
        backends.iter().find(|backend| {
            cursor += backend.weight;
            next < cursor
        })
    }

    pub async fn pick_endpoint<'a>(&self, cluster: &'a Cluster) -> Result<&'a Endpoint> {
        let healthy: Vec<&Endpoint> = cluster.endpoints.iter().filter(|ep| ep.healthy).collect();
        if healthy.is_empty() {
            return Err(TransitError::NoHealthyEndpoints(cluster.name.clone()));
        }
        let candidates = self.admissible_endpoints(cluster, &healthy);
        let idx = next_cursor(
            &self.inner.endpoint_pickers,
            &cluster.name,
            candidates.len() as u64,
        ) as usize;
        Ok(candidates[idx])
    }

    /// `healthy` minus the endpoints outlier detection has currently ejected, capped
    /// so ejection can never empty the cluster. Never returns an empty vec.
    fn admissible_endpoints<'a>(
        &self,
        cluster: &Cluster,
        healthy: &[&'a Endpoint],
    ) -> Vec<&'a Endpoint> {
        let Some(outlier) = cluster.outlier_detection.as_ref() else {
            return healthy.to_vec();
        };
        let allowance = ejection_allowance(healthy.len(), outlier);
        if allowance == 0 {
            return healthy.to_vec();
        }
        let now = Instant::now();
        let buckets = self.inner.outliers.lock().unwrap();
        let mut ejected: Vec<(usize, Instant)> = healthy
            .iter()
            .enumerate()
            .filter_map(|(idx, endpoint)| {
                buckets
                    .get(&outlier_key(&cluster.name, endpoint))
                    .and_then(|bucket| bucket.ejected_until)
                    .filter(|until| *until > now)
                    .map(|until| (idx, until))
            })
            .collect();
        drop(buckets);
        if ejected.is_empty() {
            return healthy.to_vec();
        }
        // Over the cap: keep out the endpoints furthest from recovering and re-admit
        // the rest, so the cluster keeps serving from its least-bad members.
        if ejected.len() > allowance {
            ejected.sort_by_key(|(_, until)| std::cmp::Reverse(*until));
            ejected.truncate(allowance);
        }
        let excluded: std::collections::HashSet<usize> =
            ejected.into_iter().map(|(idx, _)| idx).collect();
        healthy
            .iter()
            .enumerate()
            .filter(|(idx, _)| !excluded.contains(idx))
            .map(|(_, endpoint)| *endpoint)
            .collect()
    }

    /// Feeds one upstream result into the cluster's outlier detector. A run of
    /// `consecutive_5xx_errors` failures ejects the endpoint for a window that grows
    /// with each repeat ejection; any success clears the run.
    pub fn record_endpoint_result(&self, cluster: &Cluster, endpoint: &Endpoint, status: u16) {
        let Some(outlier) = cluster.outlier_detection.as_ref() else {
            return;
        };
        let threshold = outlier
            .consecutive_5xx_errors
            .unwrap_or(DEFAULT_CONSECUTIVE_5XX);
        if threshold == 0 {
            return;
        }
        let key = outlier_key(&cluster.name, endpoint);
        let mut buckets = self.inner.outliers.lock().unwrap();
        if status < 500 {
            if let Some(bucket) = buckets.get_mut(&key) {
                bucket.consecutive_failures = 0;
            }
            return;
        }
        let base = parse_duration(
            outlier.base_ejection_time.as_ref(),
            DEFAULT_BASE_EJECTION_TIME,
        );
        let bucket = buckets.entry(key).or_default();
        bucket.consecutive_failures += 1;
        if bucket.consecutive_failures < threshold {
            return;
        }
        bucket.consecutive_failures = 0;
        bucket.ejections = bucket.ejections.saturating_add(1);
        // saturating_mul: a long-dead endpoint must not overflow its way to a panic.
        bucket.ejected_until = Some(Instant::now() + base.saturating_mul(bucket.ejections));
    }

    pub fn check_rate_limit(&self, key: String, limit: &RateLimitPolicy) -> bool {
        let mut buckets = self.inner.rate_limits.lock().unwrap();
        let bucket = buckets.entry(key).or_insert_with(|| RateLimitBucket {
            window_started: Instant::now(),
            used: 0,
        });
        let window = Duration::from_secs(limit.window_seconds.max(1));
        if bucket.window_started.elapsed() >= window {
            bucket.window_started = Instant::now();
            bucket.used = 0;
        }
        if bucket.used >= limit.requests {
            return false;
        }
        bucket.used += 1;
        true
    }

    // Token accounting is post-hoc: admission checks the window's recorded
    // usage, and each response's usage is added once the provider reports it.
    pub fn check_token_limit(&self, key: &str, limit: &TokenLimitPolicy) -> bool {
        let mut buckets = self.inner.token_usage.lock().unwrap();
        let Some(bucket) = buckets.get_mut(key) else {
            return true;
        };
        bucket.roll_window();
        bucket.used < limit.tokens
    }

    pub fn add_token_usage(&self, key: &str, window_seconds: u64, tokens: u64) {
        // Tracks the policy's current window even if its config changed.
        let window = Duration::from_secs(window_seconds.max(1));
        let mut buckets = self.inner.token_usage.lock().unwrap();
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket {
                window_started: Instant::now(),
                window,
                used: 0,
            });
        bucket.window = window;
        bucket.roll_window();
        bucket.used = bucket.used.saturating_add(tokens);
    }

    // Err(()) means the breaker is open; there is no failure detail to carry.
    #[allow(clippy::result_unit_err)]
    pub fn try_acquire_circuit_breaker(
        &self,
        cluster: &Cluster,
    ) -> std::result::Result<Option<CircuitBreakerPermit>, ()> {
        let Some(limit) = cluster
            .circuit_breaker
            .as_ref()
            .and_then(|breaker| breaker.concurrent_request_limit())
        else {
            return Ok(None);
        };
        let mut buckets = self.inner.circuit_breakers.lock().unwrap();
        let bucket = buckets.entry(cluster.name.clone()).or_default();
        if bucket.active >= limit {
            return Err(());
        }
        bucket.active += 1;
        Ok(Some(CircuitBreakerPermit {
            state: self.clone(),
            cluster: cluster.name.clone(),
        }))
    }

    fn release_circuit_breaker(&self, cluster: &str) {
        if let Some(bucket) = self.inner.circuit_breakers.lock().unwrap().get_mut(cluster) {
            bucket.active = bucket.active.saturating_sub(1);
        }
    }

    /// Counts one request as in flight until the returned guard is dropped.
    /// Taken at the proxy entry point so requests that never reach an upstream
    /// still show up as load.
    pub fn track_request(&self) -> RequestGuard {
        let mut metrics = self.inner.metrics.lock().unwrap();
        metrics.in_flight.enter(Instant::now());
        drop(metrics);
        RequestGuard {
            state: self.clone(),
        }
    }

    /// Counts one request as in flight against a route and cluster until the
    /// returned guard is dropped.
    pub fn track_route_request(
        &self,
        namespace: &str,
        gateway: &str,
        route: &str,
        cluster: &str,
    ) -> RouteGuard {
        let key = format!("{namespace}|{gateway}|{route}|{cluster}");
        let mut metrics = self.inner.metrics.lock().unwrap();
        let counter = metrics
            .http_route_in_flight
            .entry(key.clone())
            .or_insert_with(|| HttpRouteConcurrencyCounter {
                namespace: namespace.to_string(),
                gateway: gateway.to_string(),
                route: route.to_string(),
                cluster: cluster.to_string(),
                ..HttpRouteConcurrencyCounter::default()
            });
        counter.concurrency.enter(Instant::now());
        drop(metrics);
        RouteGuard {
            state: self.clone(),
            key,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_http_request(
        &self,
        namespace: &str,
        gateway: &str,
        route: &str,
        cluster: &str,
        method: &str,
        status_code: u16,
        latency_ms: u64,
    ) {
        let mut metrics = self.inner.metrics.lock().unwrap();
        metrics.total_requests += 1;
        if status_code >= 500 {
            metrics.upstream_failures += 1;
        }
        let key = format!("{namespace}|{gateway}|{route}|{cluster}|{method}|{status_code}");
        let route_metric =
            metrics
                .http_routes
                .entry(key)
                .or_insert_with(|| HttpRouteMetricCounter {
                    namespace: namespace.to_string(),
                    gateway: gateway.to_string(),
                    route: route.to_string(),
                    cluster: cluster.to_string(),
                    method: method.to_string(),
                    status_code,
                    ..HttpRouteMetricCounter::default()
                });
        route_metric.requests += 1;
        if status_code >= 500 {
            route_metric.failures += 1;
        }
        route_metric.latency_ms_sum += latency_ms;
        for (idx, bucket) in LATENCY_BUCKETS_MS.iter().enumerate() {
            if latency_ms <= *bucket {
                route_metric.latency_ms_buckets[idx] += 1;
            }
        }
    }

    pub fn record_agent_request(
        &self,
        protocol: &str,
        route: &str,
        backend: &str,
        status: u16,
        latency_ms: u64,
    ) {
        let mut metrics = self.inner.metrics.lock().unwrap();
        metrics.total_requests += 1;
        metrics.agent_requests += 1;
        if status >= 500 {
            metrics.upstream_failures += 1;
        }
        let key = format!("{protocol}|{route}|{backend}");
        let route_metric = metrics
            .routes
            .entry(key)
            .or_insert_with(|| RouteMetricCounter {
                protocol: protocol.to_string(),
                route: route.to_string(),
                backend: backend.to_string(),
                ..RouteMetricCounter::default()
            });
        route_metric.requests += 1;
        if status >= 500 {
            route_metric.failures += 1;
        }
        route_metric.latency_ms_sum += latency_ms;
        for (idx, bucket) in LATENCY_BUCKETS_MS.iter().enumerate() {
            if latency_ms <= *bucket {
                route_metric.latency_ms_buckets[idx] += 1;
            }
        }
    }

    pub fn record_llm_usage(
        &self,
        route: &str,
        backend: &str,
        model: &str,
        prompt_tokens: u64,
        cached_prompt_tokens: u64,
        completion_tokens: u64,
    ) {
        self.record_llm_usage_full(
            route,
            backend,
            model,
            prompt_tokens,
            cached_prompt_tokens,
            0,
            completion_tokens,
            0,
            None,
            None,
            0,
            200,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_llm_usage_full(
        &self,
        route: &str,
        backend: &str,
        model: &str,
        prompt_tokens: u64,
        cached_prompt_tokens: u64,
        cache_write_tokens: u64,
        completion_tokens: u64,
        reasoning_tokens: u64,
        trace_id: Option<String>,
        span_id: Option<String>,
        latency_ms: u64,
        status_code: u16,
    ) {
        let (token_breakdown, quality) = TokenBreakdown::new(
            prompt_tokens,
            cached_prompt_tokens,
            cache_write_tokens,
            completion_tokens,
            reasoning_tokens,
        );

        let t_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let quote_res = transit_core::quote_tokens(
            model,
            transit_core::TokenCounts {
                prompt_tokens,
                cached_prompt_tokens: token_breakdown.cache_read,
                cache_write_tokens,
                completion_tokens,
            },
            transit_core::ServiceTier::Standard,
            transit_core::ContextBand::Short,
        );

        let mut metrics = self.inner.metrics.lock().unwrap();
        let key = format!("{route}|{backend}|{model}");
        {
            let counter = metrics
                .llm_usage
                .entry(key)
                .or_insert_with(|| LlmUsageMetric {
                    route: route.to_string(),
                    backend: backend.to_string(),
                    model: model.to_string(),
                    ..LlmUsageMetric::default()
                });
            counter.requests += 1;
            counter.prompt_tokens += prompt_tokens;
            counter.cached_prompt_tokens += token_breakdown.cache_read;
            counter.completion_tokens += completion_tokens;
            counter.cache_write_tokens += cache_write_tokens;
            counter.reasoning_tokens += token_breakdown.reasoning;
            if let Ok(quote) = &quote_res {
                counter.priced_requests += 1;
                counter.estimated_usd_nanos += quote.api_usd_nanos;
            }
        }

        let (
            pricing_status,
            api_usd_nanos,
            api_usd,
            chatgpt_credit_micros,
            chatgpt_credits,
            vendor,
        ) = match &quote_res {
            Ok(q) => {
                if metrics.cost_ticks.len() >= COST_TICK_CAP {
                    metrics.cost_ticks.pop_front();
                }
                metrics.cost_ticks.push_back(CostTick {
                    t_ms,
                    model: model.to_string(),
                    usd: q.api_usd_nanos as f64 / 1_000_000_000.0,
                    credits: q.chatgpt_credit_micros as f64 / 1_000_000.0,
                });
                (
                    PricingStatus::Exact,
                    Some(q.api_usd_nanos),
                    Some(q.api_usd()),
                    Some(q.chatgpt_credit_micros),
                    Some(q.chatgpt_credits()),
                    q.vendor.to_string(),
                )
            }
            Err(_) => {
                let v = if model.starts_with("claude") {
                    "anthropic"
                } else if model.starts_with("gpt-") {
                    "openai"
                } else {
                    "unknown"
                };
                (
                    PricingStatus::Unpriced,
                    None,
                    None,
                    None,
                    None,
                    v.to_string(),
                )
            }
        };

        let trace_id = trace_id.unwrap_or_else(|| format!("{t_ms:032x}"));
        let span_id = span_id.unwrap_or_else(|| format!("{:016x}", t_ms));
        let event_id = format!("evt_{:x}", t_ms % 0xffffff);

        let event = CostEvent {
            event_id,
            trace_id,
            span_id,
            parent_span_id: None,
            timestamp_ms: t_ms,
            route: route.to_string(),
            backend: backend.to_string(),
            model: model.to_string(),
            provider: vendor,
            account: backend.to_string(),
            protocol: "llm".to_string(),
            operation: "chat.completion".to_string(),
            token_breakdown,
            latency_ms,
            ttft_ms: None,
            status_code,
            data_quality: quality,
            pricing_status,
            api_usd_nanos,
            api_usd,
            chatgpt_credit_micros,
            chatgpt_credits,
            attribution_mode: AttributionMode::Direct,
            io_bytes: (prompt_tokens + completion_tokens) * 4,
            retries: 0,
        };

        if metrics.cost_events.len() >= COST_EVENT_CAP {
            metrics.cost_events.pop_front();
        }
        metrics.cost_events.push_back(event);
    }

    pub fn record_cost_event(&self, event: CostEvent) {
        let mut metrics = self.inner.metrics.lock().unwrap();
        if metrics.cost_events.len() >= COST_EVENT_CAP {
            metrics.cost_events.pop_front();
        }
        metrics.cost_events.push_back(event);
    }

    pub fn cost_events(&self) -> Vec<CostEvent> {
        let metrics = self.inner.metrics.lock().unwrap();
        metrics.cost_events.iter().cloned().collect()
    }

    pub fn cost_ticks(&self) -> Vec<CostTick> {
        self.inner
            .metrics
            .lock()
            .unwrap()
            .cost_ticks
            .iter()
            .cloned()
            .collect()
    }

    pub fn spend_ledger_summary(&self) -> SpendLedgerSummary {
        let metrics = self.inner.metrics.lock().unwrap();
        let mut total_api_usd_nanos: u128 = 0;
        let mut total_chatgpt_credit_micros: u128 = 0;
        let mut priced_requests: u64 = 0;
        let mut unpriced_requests: u64 = 0;

        let mut model_map: HashMap<String, SpendModelRow> = HashMap::new();
        let mut account_map: HashMap<String, SpendAccountRow> = HashMap::new();

        for event in &metrics.cost_events {
            if event.protocol == "llm" {
                if event.pricing_status == PricingStatus::Exact {
                    priced_requests += 1;
                    if let Some(nanos) = event.api_usd_nanos {
                        total_api_usd_nanos += nanos;
                    }
                    if let Some(credits) = event.chatgpt_credit_micros {
                        total_chatgpt_credit_micros += credits;
                    }
                } else {
                    unpriced_requests += 1;
                }

                let row = model_map
                    .entry(event.model.clone())
                    .or_insert_with(|| SpendModelRow {
                        model: event.model.clone(),
                        provider: event.provider.clone(),
                        pricing_status: event.pricing_status,
                        ..SpendModelRow::default()
                    });
                row.requests += 1;
                row.uncached_input_tokens += event.token_breakdown.input_uncached;
                row.cached_input_tokens += event.token_breakdown.cache_read;
                row.output_tokens += event.token_breakdown.total_output();

                let acct =
                    account_map
                        .entry(event.account.clone())
                        .or_insert_with(|| SpendAccountRow {
                            account: event.account.clone(),
                            provider: event.provider.clone(),
                            requests: 0,
                            api_usd: "$0".to_string(),
                            chatgpt_credits: "0".to_string(),
                        });
                acct.requests += 1;
            }
        }

        if metrics.cost_events.is_empty() {
            for metric in metrics.llm_usage.values() {
                match transit_core::quote_tokens(
                    &metric.model,
                    TokenCounts {
                        prompt_tokens: metric.prompt_tokens,
                        cached_prompt_tokens: metric.cached_prompt_tokens,
                        cache_write_tokens: 0,
                        completion_tokens: metric.completion_tokens,
                    },
                    transit_core::ServiceTier::Standard,
                    transit_core::ContextBand::Short,
                ) {
                    Ok(q) => {
                        priced_requests += metric.requests;
                        total_api_usd_nanos += q.api_usd_nanos;
                        total_chatgpt_credit_micros += q.chatgpt_credit_micros;
                        model_map.insert(
                            metric.model.clone(),
                            SpendModelRow {
                                model: metric.model.clone(),
                                provider: q.vendor.to_string(),
                                requests: metric.requests,
                                uncached_input_tokens: metric
                                    .prompt_tokens
                                    .saturating_sub(metric.cached_prompt_tokens),
                                cached_input_tokens: metric.cached_prompt_tokens,
                                output_tokens: metric.completion_tokens,
                                api_usd: q.api_usd(),
                                chatgpt_credits: q.chatgpt_credits(),
                                pricing_status: PricingStatus::Exact,
                            },
                        );
                    }
                    Err(_) => {
                        unpriced_requests += metric.requests;
                        model_map.insert(
                            metric.model.clone(),
                            SpendModelRow {
                                model: metric.model.clone(),
                                provider: "unknown".to_string(),
                                requests: metric.requests,
                                uncached_input_tokens: metric
                                    .prompt_tokens
                                    .saturating_sub(metric.cached_prompt_tokens),
                                cached_input_tokens: metric.cached_prompt_tokens,
                                output_tokens: metric.completion_tokens,
                                api_usd: "-".to_string(),
                                chatgpt_credits: "-".to_string(),
                                pricing_status: PricingStatus::Unpriced,
                            },
                        );
                    }
                }
            }
        } else {
            for row in model_map.values_mut() {
                if let Ok(q) = transit_core::quote_tokens(
                    &row.model,
                    TokenCounts {
                        prompt_tokens: row.uncached_input_tokens + row.cached_input_tokens,
                        cached_prompt_tokens: row.cached_input_tokens,
                        cache_write_tokens: 0,
                        completion_tokens: row.output_tokens,
                    },
                    transit_core::ServiceTier::Standard,
                    transit_core::ContextBand::Short,
                ) {
                    row.api_usd = q.api_usd();
                    row.chatgpt_credits = q.chatgpt_credits();
                } else {
                    row.api_usd = "-".to_string();
                    row.chatgpt_credits = "-".to_string();
                }
            }
        }

        let mut models: Vec<SpendModelRow> = model_map.into_values().collect();
        models.sort_by(|a, b| a.model.cmp(&b.model));

        let mut accounts: Vec<SpendAccountRow> = account_map.into_values().collect();
        accounts.sort_by(|a, b| a.account.cmp(&b.account));

        SpendLedgerSummary {
            total_api_usd_nanos,
            total_api_usd: format_usd_nanos(total_api_usd_nanos),
            total_chatgpt_credit_micros,
            total_chatgpt_credits: format_credit_micros(total_chatgpt_credit_micros),
            priced_requests,
            unpriced_requests,
            models,
            accounts,
        }
    }

    pub fn token_ledger_summary(&self) -> TokenLedgerSummary {
        let metrics = self.inner.metrics.lock().unwrap();
        let mut total_input_uncached: u64 = 0;
        let mut total_cache_read: u64 = 0;
        let mut total_cache_write: u64 = 0;
        let mut total_output_non_reasoning: u64 = 0;
        let mut total_reasoning: u64 = 0;
        let mut total_unclassified: u64 = 0;
        let mut complete_events: u64 = 0;
        let mut inconsistent_events: u64 = 0;

        for event in &metrics.cost_events {
            if event.protocol == "llm" {
                total_input_uncached += event.token_breakdown.input_uncached;
                total_cache_read += event.token_breakdown.cache_read;
                total_cache_write += event.token_breakdown.cache_write;
                total_output_non_reasoning += event.token_breakdown.output_non_reasoning;
                total_reasoning += event.token_breakdown.reasoning;
                total_unclassified += event.token_breakdown.unclassified;
                match event.data_quality {
                    DataQuality::Complete => complete_events += 1,
                    DataQuality::Inconsistent => inconsistent_events += 1,
                    DataQuality::Unclassified => {}
                }
            }
        }

        if metrics.cost_events.is_empty() {
            for metric in metrics.llm_usage.values() {
                let uncached = metric
                    .prompt_tokens
                    .saturating_sub(metric.cached_prompt_tokens);
                total_input_uncached += uncached;
                total_cache_read += metric.cached_prompt_tokens;
                total_output_non_reasoning += metric.completion_tokens;
                complete_events += metric.requests;
            }
        }

        let total_input = total_input_uncached + total_cache_read;
        let total_tokens =
            total_input + total_output_non_reasoning + total_reasoning + total_unclassified;
        let cache_hit_rate_pct = if total_input > 0 {
            (total_cache_read as f64 / total_input as f64) * 100.0
        } else {
            0.0
        };

        TokenLedgerSummary {
            total_input_uncached,
            total_cache_read,
            total_cache_write,
            total_output_non_reasoning,
            total_reasoning,
            total_unclassified,
            total_tokens,
            cache_hit_rate_pct,
            context_saved_tokens: total_cache_read,
            complete_events,
            inconsistent_events,
            tiers: CacheTierBreakdown {
                provider_cache_read_tokens: total_cache_read,
                provider_cache_write_tokens: total_cache_write,
                gateway_prefix_cache_tokens: 0,
                agent_context_tokens_reduced: 0,
            },
        }
    }

    pub fn efficiency_ledger_summary(&self) -> EfficiencyLedgerSummary {
        let metrics = self.inner.metrics.lock().unwrap();
        let mut mcp_calls: u64 = 0;
        let mut mcp_failures: u64 = 0;
        let mut mcp_io_bytes: u64 = 0;
        let mut a2a_calls: u64 = 0;
        let mut a2a_failures: u64 = 0;
        let mut a2a_io_bytes: u64 = 0;
        let mut total_retries: u64 = 0;
        let mut retry_overhead_tokens: u64 = 0;

        for event in &metrics.cost_events {
            total_retries += event.retries as u64;
            if event.retries > 0 {
                retry_overhead_tokens += event.token_breakdown.total() * (event.retries as u64);
            }
            match event.protocol.as_str() {
                "mcp" => {
                    mcp_calls += 1;
                    if event.status_code >= 400 {
                        mcp_failures += 1;
                    }
                    mcp_io_bytes += event.io_bytes;
                }
                "a2a" => {
                    a2a_calls += 1;
                    if event.status_code >= 400 {
                        a2a_failures += 1;
                    }
                    a2a_io_bytes += event.io_bytes;
                }
                _ => {}
            }
        }

        let mut mcp_tools: Vec<McpEfficiencyRow> = metrics
            .mcp_tools
            .values()
            .map(|m| {
                mcp_calls = mcp_calls.max(m.calls);
                mcp_failures = mcp_failures.max(m.failures);
                McpEfficiencyRow {
                    tool: m.tool.clone(),
                    backend: m.backend.clone(),
                    calls: m.calls,
                    failures: m.failures,
                    io_bytes: 0,
                    avg_latency_ms: 0,
                }
            })
            .collect();
        mcp_tools.sort_by(|a, b| a.tool.cmp(&b.tool));

        let mut a2a_methods: Vec<A2aEfficiencyRow> = metrics
            .a2a_methods
            .values()
            .map(|a| {
                a2a_calls = a2a_calls.max(a.calls);
                a2a_failures = a2a_failures.max(a.failures);
                A2aEfficiencyRow {
                    method: a.method.clone(),
                    backend: a.backend.clone(),
                    calls: a.calls,
                    failures: a.failures,
                    io_bytes: 0,
                    avg_latency_ms: 0,
                    direct_calls: a.calls,
                    rollup_calls: 0,
                }
            })
            .collect();
        a2a_methods.sort_by(|a, b| a.method.cmp(&b.method));

        EfficiencyLedgerSummary {
            mcp_calls,
            mcp_failures,
            mcp_io_bytes,
            a2a_calls,
            a2a_failures,
            a2a_io_bytes,
            total_retries,
            retry_overhead_tokens,
            mcp_tools,
            a2a_methods,
        }
    }

    pub fn optimization_ledger_summary(&self) -> OptimizationLedgerSummary {
        let token_summary = self.token_ledger_summary();
        let efficiency = self.efficiency_ledger_summary();

        let mut opportunities = Vec::new();
        if token_summary.total_cache_read > 0 {
            opportunities.push(OptimizationOpportunity {
                category: "Prompt Caching".to_string(),
                description:
                    "Provider prompt caching reused previously loaded system instructions."
                        .to_string(),
                evidence: format!(
                    "{} cache read tokens observed (hit rate {:.1}%)",
                    token_summary.total_cache_read, token_summary.cache_hit_rate_pct
                ),
                potential_tokens_saved: token_summary.total_cache_read,
                potential_usd_saved: None,
                realized: true,
            });
        } else if token_summary.total_input_uncached > 10_000 {
            opportunities.push(OptimizationOpportunity {
                category: "Prompt Caching".to_string(),
                description: "Enable prompt caching on static system prompts to reduce repetitive input billing.".to_string(),
                evidence: format!("{} uncached input tokens with 0% cache hit", token_summary.total_input_uncached),
                potential_tokens_saved: token_summary.total_input_uncached / 2,
                potential_usd_saved: None,
                realized: false,
            });
        }

        if efficiency.total_retries > 0 {
            opportunities.push(OptimizationOpportunity {
                category: "Retry Amplification".to_string(),
                description: "Exponential backoff or circuit breaking can prevent runaway duplicate request charges.".to_string(),
                evidence: format!("{} retries generated ~{} overhead tokens", efficiency.total_retries, efficiency.retry_overhead_tokens),
                potential_tokens_saved: efficiency.retry_overhead_tokens,
                potential_usd_saved: None,
                realized: false,
            });
        }

        let tokens_saved_cache = token_summary.total_cache_read;
        OptimizationLedgerSummary {
            tokens_saved_cache,
            tokens_saved_rtk: 0,
            tokens_saved_condense: 0,
            gross_potential_tokens: tokens_saved_cache,
            opportunities,
        }
    }

    pub fn record_policy_denied(&self) {
        let mut metrics = self.inner.metrics.lock().unwrap();
        metrics.total_requests += 1;
        metrics.policy_denied += 1;
    }

    pub fn record_security_decision(&self, decision: SecurityDecision) {
        let mut decisions = self.inner.security_decisions.lock().unwrap();
        if decisions.len() >= SECURITY_DECISION_CAP {
            decisions.pop_front();
        }
        decisions.push_back(decision);
    }

    pub fn security_decisions(&self) -> Vec<SecurityDecision> {
        let decisions = self.inner.security_decisions.lock().unwrap();
        decisions.iter().cloned().collect()
    }

    pub fn bind_mcp_session(&self, session_id: impl Into<String>, backend: impl Into<String>) {
        self.inner
            .mcp_sessions
            .lock()
            .unwrap()
            .bind(session_id.into(), backend.into());
    }

    pub fn mcp_session_backend(&self, session_id: &str) -> Option<String> {
        self.inner.mcp_sessions.lock().unwrap().lookup(session_id)
    }

    pub fn remove_mcp_session(&self, session_id: &str) {
        self.inner.mcp_sessions.lock().unwrap().remove(session_id);
    }

    pub fn bind_a2a_task(&self, task_id: impl Into<String>, backend: impl Into<String>) {
        self.inner
            .a2a_tasks
            .lock()
            .unwrap()
            .bind(task_id.into(), backend.into());
    }

    pub fn a2a_task_backend(&self, task_id: &str) -> Option<String> {
        self.inner.a2a_tasks.lock().unwrap().lookup(task_id)
    }

    pub fn record_a2a_method_call(&self, route: &str, backend: &str, method: &str, success: bool) {
        let mut metrics = self.inner.metrics.lock().unwrap();
        let key = format!("{route}|{backend}|{method}");
        let counter = metrics
            .a2a_methods
            .entry(key)
            .or_insert_with(|| A2aMethodMetric {
                route: route.to_string(),
                backend: backend.to_string(),
                method: method.to_string(),
                ..A2aMethodMetric::default()
            });
        counter.calls += 1;
        if !success {
            counter.failures += 1;
        }
    }

    pub fn record_mcp_tool_call(&self, route: &str, backend: &str, tool: &str, success: bool) {
        let mut metrics = self.inner.metrics.lock().unwrap();
        let key = format!("{route}|{backend}|{tool}");
        let counter = metrics
            .mcp_tools
            .entry(key)
            .or_insert_with(|| McpToolMetric {
                route: route.to_string(),
                backend: backend.to_string(),
                tool: tool.to_string(),
                ..McpToolMetric::default()
            });
        counter.calls += 1;
        if !success {
            counter.failures += 1;
        }
    }

    pub fn activation(&self) -> &Activator {
        &self.inner.activation
    }

    pub fn metrics(&self) -> ProxyMetrics {
        let mut metrics = self.inner.metrics.lock().unwrap();
        // Reading folds the elapsed time into the integrals, so a scope that
        // has been busy at a steady level since the last scrape still reports
        // the time it spent there.
        let now = Instant::now();
        let concurrency = metrics.in_flight.snapshot(now);
        let mut http_route_concurrency = metrics
            .http_route_in_flight
            .values_mut()
            .map(|route| HttpRouteConcurrencyMetric {
                namespace: route.namespace.clone(),
                gateway: route.gateway.clone(),
                route: route.route.clone(),
                cluster: route.cluster.clone(),
                concurrency: route.concurrency.snapshot(now),
            })
            .collect::<Vec<_>>();
        http_route_concurrency.sort_by(|a, b| {
            a.namespace
                .cmp(&b.namespace)
                .then_with(|| a.gateway.cmp(&b.gateway))
                .then_with(|| a.route.cmp(&b.route))
                .then_with(|| a.cluster.cmp(&b.cluster))
        });
        let metrics = &*metrics;
        let mut http_routes = metrics
            .http_routes
            .values()
            .map(|route| HttpRouteMetric {
                namespace: route.namespace.clone(),
                gateway: route.gateway.clone(),
                route: route.route.clone(),
                cluster: route.cluster.clone(),
                method: route.method.clone(),
                status_code: route.status_code,
                requests: route.requests,
                failures: route.failures,
                latency_ms_sum: route.latency_ms_sum,
                latency_ms_buckets: LATENCY_BUCKETS_MS
                    .iter()
                    .zip(route.latency_ms_buckets.iter())
                    .map(|(le, count)| LatencyBucket {
                        le: *le,
                        count: *count,
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        http_routes.sort_by(|a, b| {
            a.namespace
                .cmp(&b.namespace)
                .then_with(|| a.gateway.cmp(&b.gateway))
                .then_with(|| a.route.cmp(&b.route))
                .then_with(|| a.cluster.cmp(&b.cluster))
                .then_with(|| a.method.cmp(&b.method))
                .then_with(|| a.status_code.cmp(&b.status_code))
        });
        let mut routes = metrics
            .routes
            .values()
            .map(|route| RouteMetric {
                protocol: route.protocol.clone(),
                route: route.route.clone(),
                backend: route.backend.clone(),
                requests: route.requests,
                failures: route.failures,
                latency_ms_sum: route.latency_ms_sum,
                latency_ms_buckets: LATENCY_BUCKETS_MS
                    .iter()
                    .zip(route.latency_ms_buckets.iter())
                    .map(|(le, count)| LatencyBucket {
                        le: *le,
                        count: *count,
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        routes.sort_by(|a, b| {
            a.protocol
                .cmp(&b.protocol)
                .then_with(|| a.route.cmp(&b.route))
                .then_with(|| a.backend.cmp(&b.backend))
        });
        let mut llm_usage = metrics.llm_usage.values().cloned().collect::<Vec<_>>();
        llm_usage.sort_by(|a, b| {
            a.route
                .cmp(&b.route)
                .then_with(|| a.backend.cmp(&b.backend))
                .then_with(|| a.model.cmp(&b.model))
        });
        let mut mcp_tools = metrics.mcp_tools.values().cloned().collect::<Vec<_>>();
        mcp_tools.sort_by(|a, b| {
            a.route
                .cmp(&b.route)
                .then_with(|| a.backend.cmp(&b.backend))
                .then_with(|| a.tool.cmp(&b.tool))
        });
        let mut a2a_methods = metrics.a2a_methods.values().cloned().collect::<Vec<_>>();
        a2a_methods.sort_by(|a, b| {
            a.route
                .cmp(&b.route)
                .then_with(|| a.backend.cmp(&b.backend))
                .then_with(|| a.method.cmp(&b.method))
        });
        ProxyMetrics {
            held_activation_requests: self.inner.activation.held_requests(),
            total_requests: metrics.total_requests,
            agent_requests: metrics.agent_requests,
            policy_denied: metrics.policy_denied,
            upstream_failures: metrics.upstream_failures,
            concurrency,
            http_route_concurrency,
            http_routes,
            routes,
            llm_usage,
            mcp_tools,
            a2a_methods,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use transit_core::{Cluster, Listener, ListenerProtocol, PathMatch, Route, RouteMatch, VirtualHost};

    fn test_cluster(
        name: &str,
        endpoints: Vec<Endpoint>,
        outlier_detection: Option<OutlierDetectionConfig>,
    ) -> Cluster {
        Cluster {
            name: name.into(),
            endpoints,
            http2: false,
            tls: None,
            circuit_breaker: None,
            outlier_detection,
        }
    }

    fn endpoint(address: &str) -> Endpoint {
        Endpoint {
            address: address.into(),
            port: 8080,
            healthy: true,
            node_name: None,
        }
    }

    fn valid_config(version: &str) -> RuntimeConfig {
        RuntimeConfig {
            version: version.into(),
            listeners: vec![Listener {
                name: "http".into(),
                bind: "0.0.0.0:80".parse().unwrap(),
                protocol: ListenerProtocol::Http,
                virtual_hosts: vec![VirtualHost {
                    name: "wildcard".into(),
                    domains: vec!["*".into()],
                    routes: vec![Route {
                        name: "default".into(),
                        matches: vec![RouteMatch {
                            path: PathMatch::Prefix("/".into()),
                            headers: vec![],
                        }],
                        weighted_clusters: vec![WeightedCluster {
                            name: "backend".into(),
                            weight: 100,
                        }],
                    }],
                }],
                tls_secret: None,
                security: Default::default(),
            }],
            clusters: vec![Cluster {
                name: "backend".into(),
                endpoints: vec![Endpoint {
                    address: "127.0.0.1".into(),
                    port: 8080,
                    healthy: true,
                    node_name: None,
                }],
                http2: false,
                tls: None,
                circuit_breaker: None,
                outlier_detection: None,
            }],
            secrets: vec![],
            providers: vec![],
            backends: vec![],
            routes: vec![],
            policies: vec![],
        }
    }

    #[test]
    fn in_flight_tracks_guard_lifetime() {
        let state = ProxyState::new();
        assert_eq!(state.metrics().concurrency.in_flight, 0);

        let first = state.track_request();
        let second = state.track_request();
        assert_eq!(state.metrics().concurrency.in_flight, 2);

        drop(first);
        assert_eq!(state.metrics().concurrency.in_flight, 1);
        drop(second);
        assert_eq!(state.metrics().concurrency.in_flight, 0);
    }

    #[test]
    fn route_in_flight_is_scoped_to_route_and_cluster() {
        let state = ProxyState::new();
        let orders = state.track_route_request("app", "public", "orders", "orders-v1");
        let _reviews = state.track_route_request("app", "public", "reviews", "reviews-v1");

        let by_route = |metrics: &ProxyMetrics, route: &str| {
            metrics
                .http_route_concurrency
                .iter()
                .find(|entry| entry.route == route)
                .map(|entry| entry.concurrency.in_flight)
                .unwrap_or_default()
        };

        let metrics = state.metrics();
        assert_eq!(by_route(&metrics, "orders"), 1);
        assert_eq!(by_route(&metrics, "reviews"), 1);

        drop(orders);
        let metrics = state.metrics();
        assert_eq!(by_route(&metrics, "orders"), 0);
        // The series stays published at zero so a scrape can tell "idle" apart
        // from "route no longer exists".
        assert_eq!(by_route(&metrics, "reviews"), 1);
    }

    // The integral is what an autoscaler reads through rate(); a gauge sampled
    // between scrapes would report zero for a request that started and finished
    // in the gap.
    #[test]
    fn request_seconds_accumulate_while_in_flight() {
        let state = ProxyState::new();
        assert_eq!(state.metrics().concurrency.seconds_total, 0.0);

        let guard = state.track_request();
        std::thread::sleep(Duration::from_millis(20));
        let held = state.metrics().concurrency.seconds_total;
        assert!(
            held > 0.0,
            "seconds_total = {held}, want > 0 while in flight"
        );

        drop(guard);
        let after_drop = state.metrics().concurrency.seconds_total;
        assert!(after_drop >= held);

        // An idle gateway must stop accumulating, otherwise rate() reports load
        // that is not there.
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(state.metrics().concurrency.seconds_total, after_drop);
    }

    #[test]
    fn mcp_session_bindings_are_capped() {
        let state = ProxyState::new();
        for i in 0..=BINDING_CAP {
            state.bind_mcp_session(format!("session-{i}"), "backend");
        }
        // The idlest binding (the first inserted) was evicted to stay at cap.
        assert_eq!(state.mcp_session_backend("session-0"), None);
        assert_eq!(
            state.mcp_session_backend(&format!("session-{BINDING_CAP}")),
            Some("backend".to_string())
        );
    }

    #[test]
    fn a2a_task_bindings_round_trip() {
        let state = ProxyState::new();
        state.bind_a2a_task("task-1", "planner");
        assert_eq!(
            state.a2a_task_backend("task-1"),
            Some("planner".to_string())
        );
        assert_eq!(state.a2a_task_backend("task-2"), None);
    }

    #[test]
    fn mcp_tool_calls_aggregate_per_tool() {
        let state = ProxyState::new();
        state.record_mcp_tool_call("mcp", "mcp-a", "search", true);
        state.record_mcp_tool_call("mcp", "mcp-a", "search", false);
        state.record_mcp_tool_call("mcp", "mcp-b", "calendar", true);

        let metrics = state.metrics();
        assert_eq!(metrics.mcp_tools.len(), 2);
        let search = metrics
            .mcp_tools
            .iter()
            .find(|tool| tool.tool == "search")
            .unwrap();
        assert_eq!(search.calls, 2);
        assert_eq!(search.failures, 1);
    }

    #[tokio::test]
    async fn apply_config_updates_readiness_and_conflicts() {
        let state = ProxyState::new();
        state.apply_config(valid_config("ok")).unwrap();

        let readiness = state.readiness();
        assert!(readiness.ready);
        assert_eq!(readiness.version, "xds=ok");
        assert!(readiness.conflicts.is_empty());

        let mut invalid = valid_config("bad");
        invalid.clusters.clear();
        let conflicts = state.apply_config(invalid).unwrap_err();

        let readiness = state.readiness();
        assert!(!readiness.ready);
        assert_eq!(readiness.conflicts, conflicts);
        assert_eq!(readiness.conflicts[0].kind, "missing-cluster");
    }

    #[tokio::test]
    async fn weighted_cluster_picker_is_deterministic() {
        let state = ProxyState::new();
        let clusters = vec![
            WeightedCluster {
                name: "a".into(),
                weight: 2,
            },
            WeightedCluster {
                name: "b".into(),
                weight: 1,
            },
        ];
        let mut names = Vec::new();

        for _ in 0..6 {
            names.push(
                state
                    .pick_cluster("default", &clusters)
                    .await
                    .unwrap()
                    .name
                    .clone(),
            );
        }

        assert_eq!(names, ["a", "a", "b", "a", "a", "b"]);
    }

    #[test]
    fn circuit_breaker_enforces_concurrent_limit() {
        let state = ProxyState::new();
        let cluster = Cluster {
            name: "backend".into(),
            endpoints: vec![],
            http2: false,
            tls: None,
            circuit_breaker: Some(transit_core::CircuitBreakerConfig {
                max_connections: None,
                http1_max_pending_requests: None,
                http2_max_requests: Some(1),
                max_requests_per_connection: None,
                max_retries: None,
            }),
            outlier_detection: None,
        };

        let permit = state
            .try_acquire_circuit_breaker(&cluster)
            .expect("first request should pass")
            .expect("configured circuit breaker should return a permit");
        assert!(state.try_acquire_circuit_breaker(&cluster).is_err());
        drop(permit);
        assert!(state
            .try_acquire_circuit_breaker(&cluster)
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn endpoint_picker_skips_unhealthy_endpoints() {
        let state = ProxyState::new();
        let endpoints = vec![
            Endpoint {
                address: "10.0.0.1".into(),
                port: 8080,
                healthy: false,
                node_name: None,
            },
            Endpoint {
                address: "10.0.0.2".into(),
                port: 8080,
                healthy: true,
                node_name: None,
            },
        ];

        let cluster = test_cluster("backend", endpoints, None);
        let endpoint = state.pick_endpoint(&cluster).await.unwrap();
        assert_eq!(endpoint.address, "10.0.0.2");

        let unhealthy = test_cluster(
            "backend",
            vec![Endpoint {
                address: "10.0.0.3".into(),
                port: 8080,
                healthy: false,
                node_name: None,
            }],
            None,
        );
        assert!(state.pick_endpoint(&unhealthy).await.is_err());
    }

    #[tokio::test]
    async fn round_robin_cursors_are_per_cluster() {
        let state = ProxyState::new();
        let a = test_cluster("a", vec![endpoint("10.0.0.1"), endpoint("10.0.0.2")], None);
        let b = test_cluster("b", vec![endpoint("10.1.0.1"), endpoint("10.1.0.2")], None);

        // Interleaving two clusters must not advance the other's cursor: each still
        // alternates over its own endpoints. A shared counter made `a` return
        // 10.0.0.1 every time here.
        let mut picked = Vec::new();
        for _ in 0..4 {
            picked.push(state.pick_endpoint(&a).await.unwrap().address.clone());
            let _ = state.pick_endpoint(&b).await.unwrap();
        }

        assert_eq!(picked, ["10.0.0.1", "10.0.0.2", "10.0.0.1", "10.0.0.2"]);
    }

    #[tokio::test]
    async fn outlier_detection_ejects_after_consecutive_failures() {
        let state = ProxyState::new();
        let cluster = test_cluster(
            "backend",
            vec![endpoint("10.0.0.1"), endpoint("10.0.0.2")],
            Some(OutlierDetectionConfig {
                consecutive_5xx_errors: Some(2),
                interval: None,
                base_ejection_time: Some("30s".into()),
                // 50% of two endpoints: exactly one may be ejected.
                max_ejection_percent: Some(50),
                min_health_percent: None,
            }),
        );
        let bad = &cluster.endpoints[0];

        state.record_endpoint_result(&cluster, bad, 503);
        // One failure is below the threshold, so the endpoint still serves.
        assert!(state
            .admissible_endpoints(&cluster, &cluster.endpoints.iter().collect::<Vec<_>>())
            .iter()
            .any(|ep| ep.address == "10.0.0.1"));

        state.record_endpoint_result(&cluster, bad, 503);
        let admissible =
            state.admissible_endpoints(&cluster, &cluster.endpoints.iter().collect::<Vec<_>>());
        assert_eq!(admissible.len(), 1);
        assert_eq!(admissible[0].address, "10.0.0.2");

        // Every pick now avoids the ejected endpoint.
        for _ in 0..4 {
            assert_eq!(
                state.pick_endpoint(&cluster).await.unwrap().address,
                "10.0.0.2"
            );
        }
    }

    #[test]
    fn outlier_detection_success_clears_the_failure_run() {
        let state = ProxyState::new();
        let cluster = test_cluster(
            "backend",
            vec![endpoint("10.0.0.1"), endpoint("10.0.0.2")],
            Some(OutlierDetectionConfig {
                consecutive_5xx_errors: Some(2),
                interval: None,
                base_ejection_time: None,
                max_ejection_percent: Some(50),
                min_health_percent: None,
            }),
        );
        let flaky = &cluster.endpoints[0];

        state.record_endpoint_result(&cluster, flaky, 503);
        state.record_endpoint_result(&cluster, flaky, 200);
        state.record_endpoint_result(&cluster, flaky, 503);

        // Failures were not consecutive, so nothing is ejected.
        assert_eq!(
            state
                .admissible_endpoints(&cluster, &cluster.endpoints.iter().collect::<Vec<_>>())
                .len(),
            2
        );
    }

    #[test]
    fn ejection_never_empties_a_cluster() {
        let cfg = OutlierDetectionConfig {
            consecutive_5xx_errors: Some(1),
            interval: None,
            base_ejection_time: None,
            max_ejection_percent: Some(100),
            min_health_percent: Some(50),
        };
        // min_health_percent caps the allowance below max_ejection_percent.
        assert_eq!(ejection_allowance(4, &cfg), 2);
        // Envoy's 10% default cannot eject either of two endpoints.
        assert_eq!(
            ejection_allowance(
                2,
                &OutlierDetectionConfig {
                    consecutive_5xx_errors: None,
                    interval: None,
                    base_ejection_time: None,
                    max_ejection_percent: None,
                    min_health_percent: None,
                }
            ),
            0
        );
    }

    #[test]
    fn outlier_durations_parse_xds_and_yaml_forms() {
        let fallback = Duration::from_secs(30);
        assert_eq!(
            parse_duration(Some(&"10s".to_string()), fallback),
            Duration::from_secs(10)
        );
        assert_eq!(
            parse_duration(Some(&"0.500000000s".to_string()), fallback),
            Duration::from_millis(500)
        );
        assert_eq!(
            parse_duration(Some(&"250ms".to_string()), fallback),
            Duration::from_millis(250)
        );
        // Unparseable, zero, and absent values all fall back rather than disabling
        // ejection with a zero-length window.
        assert_eq!(
            parse_duration(Some(&"soon".to_string()), fallback),
            fallback
        );
        assert_eq!(parse_duration(Some(&"0s".to_string()), fallback), fallback);
        assert_eq!(parse_duration(None, fallback), fallback);
    }

    #[test]
    fn records_http_route_metrics() {
        let state = ProxyState::new();

        state.record_http_request("app", "public", "default", "reviews", "GET", 200, 12);
        state.record_http_request("app", "public", "default", "reviews", "GET", 502, 260);

        let metrics = state.metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.upstream_failures, 1);
        assert_eq!(metrics.http_routes.len(), 2);
        let route = metrics
            .http_routes
            .iter()
            .find(|metric| metric.status_code == 502)
            .expect("missing 502 route metric");
        assert_eq!(route.namespace, "app");
        assert_eq!(route.gateway, "public");
        assert_eq!(route.route, "default");
        assert_eq!(route.cluster, "reviews");
        assert_eq!(route.method, "GET");
        assert_eq!(route.requests, 1);
        assert_eq!(route.failures, 1);
        assert_eq!(route.latency_ms_sum, 260);
        assert_eq!(route.latency_ms_buckets[6].count, 1);
    }

    #[test]
    fn records_and_bounds_security_decisions() {
        let state = ProxyState::new();
        let initial = state.security_decisions();
        assert!(initial.is_empty());

        let custom = SecurityDecision {
            event_id: "sec-test".into(),
            trace_id: "trace-test".into(),
            timestamp: "2026-09-04T08:30:00Z".into(),
            listener: "http-8080".into(),
            route: "chat".into(),
            backend: "codex".into(),
            protocol: "llm".into(),
            actor: "agent:test".into(),
            principal: "test-principal".into(),
            authn_method: "jwt".into(),
            enforcement_point: "PEP: Custom".into(),
            policy_id: "test-policy".into(),
            resource_kind: "route".into(),
            resource_id: "/test".into(),
            decision: "allowed".into(),
            reason_code: "authz.allow".into(),
            status_code: 200,
            latency_ms: 5,
            evidence_hash: "hash".into(),
            redacted_attributes: BTreeMap::new(),
        };
        state.record_security_decision(custom.clone());
        let updated = state.security_decisions();
        assert_eq!(updated.last().unwrap().event_id, "sec-test");
    }
}
