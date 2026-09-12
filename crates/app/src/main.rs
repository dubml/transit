use clap::Parser;
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::Sampler;
use opentelemetry_sdk::Resource;
use std::collections::{BTreeSet, HashMap};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time;
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use transit_core::{
    AuthPolicy, ConfigStore, RouterIdentity, RuntimeMode, SecretKeyReference, DEFAULT_CLUSTER_ID,
    DEFAULT_DNS_DOMAIN,
};
use transit_proxy::{ProxyServer, ProxyState};
use transit_ui::UiServer;
use transit_xds::{BootstrapConfig, XdsClient, XdsClientConfig};

mod local_config;
mod mode;

#[derive(Debug, Parser)]
#[command(name = "transit")]
#[command(about = "Pure Rust north-south proxy for Dubbo Gateway API traffic")]
struct Args {
    /// Deployment environment; never inferred from kubeconfig or service-account files.
    #[arg(long, env = "TRANSIT_MODE", default_value = "standalone")]
    mode: RuntimeMode,

    #[arg(long, env = "TRANSIT_XDS_ADDRESS", default_value = "")]
    xds_address: String,

    #[arg(long, env = "TRANSIT_XDS_ENABLED")]
    xds_enabled: Option<bool>,

    /// Gateway namespace/name used by the independent control plane.
    #[arg(long, env = "TRANSIT_GATEWAY")]
    gateway: Option<String>,

    #[arg(long, env = "TRANSIT_XDS_ROOT_CA")]
    xds_root_ca: Option<PathBuf>,

    /// Projected ServiceAccount token with the transit-xds audience.
    #[arg(long, env = "TRANSIT_XDS_TOKEN_FILE")]
    xds_token_file: Option<PathBuf>,

    #[arg(long, env = "TRANSIT_HTTP_ADDR", default_value_t = SocketAddr::from(([0, 0, 0, 0], transit_proxy::access_settings::DEFAULT_HTTP_PORT)))]
    http_addr: SocketAddr,

    #[arg(long, env = "TRANSIT_UI_ADDR", default_value_t = SocketAddr::from(([0, 0, 0, 0], transit_core::UI_PORT)))]
    ui_addr: SocketAddr,

    /// Directory containing gateway-managed OAuth accounts.
    #[arg(long, env = "TRANSIT_LLM_ACCOUNTS_DIR")]
    llm_accounts_dir: Option<PathBuf>,

    /// Initial management key; a key saved in Configuration takes precedence.
    #[arg(long, env = "TRANSIT_LLM_ADMIN_TOKEN", hide_env_values = true)]
    llm_admin_token: Option<String>,

    /// Persistent access/authentication settings edited by the Configuration page.
    #[arg(long, env = "TRANSIT_ACCESS_CONFIG")]
    access_config: Option<PathBuf>,

    #[arg(long, env = "TRANSIT_METRICS_ENABLED", default_value_t = true)]
    metrics_enabled: bool,

    // Should be <= the pod's terminationGracePeriodSeconds, or Kubernetes SIGKILLs
    // the process mid-drain and the graceful shutdown buys nothing.
    #[arg(long, env = "TRANSIT_DRAIN_TIMEOUT_SECONDS", default_value_t = 30)]
    drain_timeout_seconds: u64,

    #[arg(long, env = "TRANSIT_BOOTSTRAP")]
    bootstrap: Option<PathBuf>,

    #[arg(long, env = "TRANSIT_STATIC_CONFIG")]
    static_config: Option<PathBuf>,

    #[arg(long, env = "TRANSIT_OTEL_ENDPOINT")]
    otel_endpoint: Option<String>,

    #[arg(long, env = "TRANSIT_OTEL_SERVICE_NAME", default_value = "transit")]
    otel_service_name: String,

    #[arg(
        long,
        env = "TRANSIT_OTEL_SAMPLING_PERCENTAGE",
        default_value_t = 100.0
    )]
    otel_sampling_percentage: f64,

    #[arg(long, env = "TRANSIT_OTEL_TAGS")]
    otel_tags: Option<String>,

    #[arg(long, env = "TRANSIT_LISTENER_NAMES", value_delimiter = ',')]
    listener_names: Vec<String>,

    #[arg(long, env = "POD_NAME", default_value = "transit")]
    pod_name: String,

    #[arg(long, env = "POD_NAMESPACE", default_value = "dubbo-system")]
    namespace: String,

    #[arg(long, env = "INSTANCE_IP", default_value = "127.0.0.1")]
    pod_ip: String,

    #[arg(long, env = "KUBE_NODE_NAME")]
    node_name: Option<String>,

    #[arg(long, env = "DUBBO_META_CLUSTER_ID", default_value = DEFAULT_CLUSTER_ID)]
    cluster_id: String,

    #[arg(long, env = "DOMAIN_SUFFIX", default_value = DEFAULT_DNS_DOMAIN)]
    dns_domain: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Quote observed tokens against published OpenAI API USD and ChatGPT credit tables.
    /// Does not send traffic.
    Ledger {
        #[command(subcommand)]
        action: Option<LedgerAction>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 0)]
        prompt_tokens: u64,
        #[arg(long, default_value_t = 0)]
        cached_tokens: u64,
        #[arg(long, default_value_t = 0)]
        cache_write_tokens: u64,
        #[arg(long, default_value_t = 0)]
        completion_tokens: u64,
        /// standard | fast | flex | batch
        #[arg(long, default_value = "standard")]
        tier: String,
        /// short | long
        #[arg(long, default_value = "short")]
        context: String,
        #[arg(long)]
        json: bool,
        /// Treasury country, currency code, or "United States-Dollar"
        #[arg(long)]
        country: Option<String>,
    },
}

#[derive(Debug, clap::Subcommand)]
enum LedgerAction {
    /// Scan local Claude Code and Codex CLI session logs. No network.
    Local {
        #[arg(long)]
        json: bool,
        #[arg(long, env = "CLAUDE_CONFIG_DIR")]
        claude_home: Option<String>,
        #[arg(long, env = "CODEX_HOME")]
        codex_home: Option<String>,
    },
}

fn run_ledger(
    model: String,
    tokens: transit_core::TokenCounts,
    tier: String,
    context: String,
    json: bool,
    country: Option<String>,
) -> std::io::Result<()> {
    use transit_core::{quote_tokens, ContextBand, ServiceTier};
    let tier = match tier.to_ascii_lowercase().as_str() {
        "standard" => ServiceTier::Standard,
        "fast" | "priority" => ServiceTier::Fast,
        "flex" => ServiceTier::Flex,
        "batch" => ServiceTier::Batch,
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown tier {other}"),
            ));
        }
    };
    let context = match context.to_ascii_lowercase().as_str() {
        "short" => ContextBand::Short,
        "long" => ContextBand::Long,
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown context {other}"),
            ));
        }
    };
    let quote = quote_tokens(&model, tokens, tier, context)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;
    if json {
        let payload = if let Some(country) = country.as_deref() {
            let fx =
                transit_core::convert_usd_nanos(quote.api_usd_nanos, country).map_err(|err| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string())
                })?;
            serde_json::json!({ "quote": quote, "fx": fx })
        } else {
            serde_json::to_value(&quote).map_err(|err| std::io::Error::other(err.to_string()))?
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&payload)
                .map_err(|err| std::io::Error::other(err.to_string()))?
        );
        return Ok(());
    }
    println!("model\t{}", quote.model);
    println!("tier\t{:?}", quote.tier);
    println!("context\t{:?}", quote.context);
    println!("uncached_input_tokens\t{}", quote.uncached_prompt_tokens);
    println!("api_usd\t{}", quote.api_usd());
    println!("chatgpt_credits\t{}", quote.chatgpt_credits());
    println!(
        "chatgpt_credits_complete\t{}",
        quote.chatgpt_credits_complete
    );
    println!("api_usd_source\t{}", quote.api_usd_source);
    println!("chatgpt_credits_source\t{}", quote.chatgpt_credits_source);
    println!("chatgpt_fast_source\t{}", quote.chatgpt_fast_source);
    println!("rate_card_as_of\t{}", quote.rate_card_as_of);
    println!("formula\t{}", quote.formula);
    for item in &quote.line_items {
        println!(
            "line\t{}\ttokens={}\tusd={}\tcredits={}",
            item.component,
            item.tokens,
            transit_core::format_usd_nanos(item.api_usd_nanos),
            item.chatgpt_credit_micros
                .map(transit_core::format_credit_micros)
                .unwrap_or_else(|| "unpublished".into())
        );
    }
    if let Some(country) = country {
        let fx = transit_core::convert_usd_nanos(quote.api_usd_nanos, &country).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string())
        })?;
        println!("fx_country\t{}", fx.country);
        println!("fx_currency\t{}", fx.currency_code);
        println!("fx_units_per_usd\t{}", fx.units_per_usd);
        println!("fx_amount\t{}", fx.amount);
        println!("fx_source\t{}", fx.source);
        println!("fx_as_of\t{}", fx.as_of);
    }
    Ok(())
}

fn run_local_ledger(
    json: bool,
    claude_home: Option<String>,
    codex_home: Option<String>,
) -> std::io::Result<()> {
    let mut paths = transit_core::LocalScanPaths::from_env();
    if let Some(value) = claude_home {
        paths.claude_roots = transit_core::LocalScanPaths::parse_roots(&value);
    }
    if let Some(value) = codex_home {
        paths.codex_roots = transit_core::LocalScanPaths::parse_roots(&value);
    }
    let report = transit_core::scan_local_usage(&paths);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|err| std::io::Error::other(err.to_string()))?
        );
        return Ok(());
    }
    println!("claude_files\t{}", report.claude_files);
    println!("codex_files\t{}", report.codex_files);
    println!("skipped_files\t{}", report.skipped_files);
    println!("source\tmodel\trequests\tinput\tcache_read\tcache_write\toutput\ttotal");
    for row in &report.rows {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.source,
            row.model,
            row.requests,
            row.prompt_tokens,
            row.cached_prompt_tokens,
            row.cache_write_tokens,
            row.completion_tokens,
            row.total_tokens
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if let Some(Command::Ledger {
        action,
        model,
        prompt_tokens,
        cached_tokens,
        cache_write_tokens,
        completion_tokens,
        tier,
        context,
        json,
        country,
    }) = args.command
    {
        if let Some(LedgerAction::Local {
            json,
            claude_home,
            codex_home,
        }) = action
        {
            return run_local_ledger(json, claude_home, codex_home);
        }
        let model = model.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "ledger quote requires --model",
            )
        })?;
        return run_ledger(
            model,
            transit_core::TokenCounts {
                prompt_tokens,
                cached_prompt_tokens: cached_tokens,
                cache_write_tokens,
                completion_tokens,
            },
            tier,
            context,
            json,
            country,
        );
    }

    // kube's rustls stack disables a default crypto backend; install one
    // before the Secret resolver creates its first Kubernetes TLS client.
    let _ = rustls_kube::crypto::ring::default_provider().install_default();

    let mut args = args;
    if let Some(path) = args.bootstrap.clone() {
        let bootstrap = BootstrapConfig::load(path)
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        apply_bootstrap(&mut args, bootstrap);
    }
    let otel_enabled = init_tracing(
        args.otel_endpoint.as_deref(),
        &args.otel_service_name,
        args.otel_sampling_percentage,
        args.otel_tags.as_deref(),
    )?;

    let startup = mode::StartupPlan::from_args(&args)
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
    let identity = RouterIdentity {
        pod_name: args.pod_name,
        namespace: args.namespace.clone(),
        pod_ip: args.pod_ip,
        node_name: args.node_name,
        cluster_id: args.cluster_id,
        dns_domain: args.dns_domain,
    };

    info!(node_id = %identity.node_id(), "starting transit router proxy");

    // Startup selects exactly one routing configuration owner.
    let store = Arc::new(ConfigStore::new());
    let state = ProxyState::with_store(store.clone());
    let access_base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    let access_path = args
        .access_config
        .clone()
        .unwrap_or_else(|| access_base.join("transit/access.json"));
    let access_path = if access_path.is_absolute() {
        access_path
    } else {
        std::env::current_dir()?.join(access_path)
    };
    let account_dir = args
        .llm_accounts_dir
        .clone()
        .unwrap_or_else(|| access_base.join("transit/accounts"));
    let account_dir = if account_dir.is_absolute() {
        account_dir
    } else {
        std::env::current_dir()?.join(account_dir)
    };
    let access_defaults = transit_proxy::access_settings::AccessConfig {
        host: args.http_addr.ip().to_string(),
        port: args.http_addr.port(),
        auth_dir: account_dir.display().to_string(),
        api_keys: Vec::new(),
        tls: Default::default(),
        remote_management: transit_proxy::access_settings::RemoteManagement {
            allow_remote: args.llm_admin_token.is_some(),
            secret_key: args.llm_admin_token.clone().unwrap_or_default(),
            ..Default::default()
        },
    };
    let access = if args.mode == RuntimeMode::Standalone {
        state
            .access_settings()
            .configure(access_path, access_defaults)?
    } else {
        state
            .access_settings()
            .configure_read_only(args.access_config.as_deref(), access_defaults)?
    };
    let http_addr = SocketAddr::new(
        access.host.parse().map_err(std::io::Error::other)?,
        access.port,
    );
    if args.mode == RuntimeMode::Standalone || args.llm_accounts_dir.is_some() {
        configure_llm_accounts(
            &state,
            args.ui_addr,
            Some(&transit_proxy::access_settings::expand_path(
                &access.auth_dir,
            )?),
            args.llm_admin_token.as_deref(),
        )?;
    }

    if let Some(endpoint) = startup.xds_endpoint {
        let mut xds = XdsClient::new(XdsClientConfig {
            endpoint,
            identity,
            listener_names: args.listener_names,
            reconnect_delay: Duration::from_secs(10),
        });
        if let Some(gateway) = args.gateway {
            xds = xds.with_gateway_identity(gateway);
        }
        if let (Some(ca), Some(token)) = (args.xds_root_ca, args.xds_token_file) {
            xds = xds.with_service_account_credentials(ca, token);
        }
        let xds_store = store.clone();
        tokio::spawn(async move {
            if let Err(err) = xds.run(xds_store).await {
                error!(%err, "xDS client exited");
            }
        });
    } else {
        info!("xDS client disabled");
    }

    let local_configuration = match args.static_config.clone() {
        Some(path) => Some(local_config::LocalConfiguration::load(path, &state).await?),
        None => None,
    };

    if args.mode == RuntimeMode::Kubernetes {
        tokio::spawn(sync_referenced_secrets(
            state.clone(),
            args.namespace.clone(),
        ));
    }

    let proxy = ProxyServer::new(state.clone());
    let access_log_proxy = proxy.clone();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let config_watcher = local_configuration.map(|configuration| {
        tokio::spawn(configuration.watch(state.clone(), shutdown_requested(shutdown_rx.clone())))
    });
    if args.mode == RuntimeMode::Kubernetes {
        state.require_configured_listeners();
    }
    let ui = UiServer::new(state, http_addr, args.metrics_enabled)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_runtime_mode(startup.runtime);
    let proxy_shutdown = shutdown_rx.clone();
    let mut proxy_task = tokio::spawn(async move {
        if args.mode == RuntimeMode::Kubernetes {
            proxy
                .serve_configured_with_shutdown(
                    Duration::from_secs(args.drain_timeout_seconds),
                    shutdown_requested(proxy_shutdown),
                )
                .await
        } else {
            proxy
                .serve_with_shutdown(http_addr, shutdown_requested(proxy_shutdown))
                .await
        }
    });
    let mut ui_task =
        tokio::spawn(ui.serve_with_shutdown(args.ui_addr, shutdown_requested(shutdown_rx)));

    tokio::select! {
        result = &mut proxy_task => result.unwrap_or_else(|err| Err(std::io::Error::other(err)))?,
        result = &mut ui_task => result.unwrap_or_else(|err| Err(std::io::Error::other(err)))?,
        _ = termination_signal() => {
            let drain = Duration::from_secs(args.drain_timeout_seconds);
            info!(drain_timeout = ?drain, "received shutdown signal, draining in-flight requests");
            // Both listeners stop accepting; in-flight requests get until the drain
            // timeout to finish, after which the process exits regardless so a stuck
            // upstream cannot outlive the pod's termination grace period.
            let _ = shutdown_tx.send(true);
            if time::timeout(drain, async {
                let _ = tokio::join!(&mut proxy_task, &mut ui_task);
            })
            .await
            .is_err()
            {
                warn!("drain timeout elapsed with requests still in flight");
            }
        }
    }

    access_log_proxy.shutdown_access_logs().await;
    if let Some(watcher) = config_watcher {
        watcher.abort();
        let _ = watcher.await;
    }

    if otel_enabled {
        opentelemetry::global::shutdown_tracer_provider();
    }

    Ok(())
}

/// Resolves on SIGTERM (what Kubernetes sends first) or SIGINT (Ctrl-C).
fn configure_llm_accounts(
    state: &ProxyState,
    ui_addr: SocketAddr,
    directory: Option<&std::path::Path>,
    token: Option<&str>,
) -> std::io::Result<()> {
    if state.access_settings().configured() {
        return state.llm_accounts().configure_local(
            directory
                .ok_or_else(|| std::io::Error::other("Authentication directory unavailable"))?,
        );
    }
    let local = ui_addr.ip().is_loopback();
    if !local && directory.is_none() && token.is_none() {
        return Ok(());
    }
    if !local && token.is_none() {
        return Err(std::io::Error::other(
            "TRANSIT_LLM_ADMIN_TOKEN is required for OAuth management on a non-loopback UI",
        ));
    }
    let directory = match directory {
        Some(path) => path.to_path_buf(),
        None => {
            let base = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
                })
                .ok_or_else(|| {
                    std::io::Error::other(
                        "Set TRANSIT_LLM_ACCOUNTS_DIR when no home directory is available",
                    )
                })?;
            base.join("transit/accounts")
        }
    };
    if let Some(token) = token {
        state
            .llm_accounts()
            .configure(&directory, token.to_string())?;
    } else {
        state.llm_accounts().configure_local(&directory)?;
    }
    info!(path = %directory.display(), local_session = token.is_none(), "OAuth account storage enabled");
    Ok(())
}

async fn termination_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(stream) => stream,
            Err(err) => {
                error!(%err, "failed installing SIGTERM handler; falling back to SIGINT only");
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn shutdown_requested(mut rx: watch::Receiver<bool>) {
    loop {
        if *rx.borrow() {
            return;
        }
        if rx.changed().await.is_err() {
            return;
        }
    }
}

async fn sync_referenced_secrets(state: ProxyState, namespace: String) {
    let mut client: Option<Client> = None;
    let mut ticker = time::interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        let references = referenced_secrets(&state);
        if references.is_empty() {
            state.replace_credentials(HashMap::new());
            continue;
        }
        let kube = match &client {
            Some(client) => client.clone(),
            None => match Client::try_default().await {
                Ok(value) => {
                    client = Some(value.clone());
                    value
                }
                Err(err) => {
                    warn!(%err, "cannot initialize Kubernetes Secret resolver");
                    continue;
                }
            },
        };
        let mut values = HashMap::new();
        for reference in references {
            if reference.namespace != namespace {
                warn!(
                    secret_namespace = %reference.namespace,
                    secret_name = %reference.name,
                    gateway_namespace = %namespace,
                    "cross-namespace credential reference rejected"
                );
                continue;
            }
            let api = Api::<Secret>::namespaced(kube.clone(), &namespace);
            match api.get(&reference.name).await {
                Ok(secret) => {
                    if let Some(value) = secret
                        .data
                        .as_ref()
                        .and_then(|data| data.get(&reference.key))
                        .and_then(|value| String::from_utf8(value.0.clone()).ok())
                    {
                        values.insert(reference, value);
                    } else {
                        warn!(secret = %reference.name, key = %reference.key, "credential key missing");
                    }
                }
                Err(err) => warn!(secret = %reference.name, %err, "credential Secret unavailable"),
            }
        }
        state.replace_credentials(values);
    }
}

fn referenced_secrets(state: &ProxyState) -> BTreeSet<SecretKeyReference> {
    let snapshot = state.snapshot();
    let mut references = BTreeSet::new();
    for provider in snapshot.providers() {
        if let Some(reference) = &provider.credential_ref {
            references.insert(reference.clone());
        }
    }
    for policy in snapshot.policies() {
        if let Some(AuthPolicy::ApiKey {
            secret_ref: Some(reference),
            ..
        }) = &policy.auth
        {
            references.insert(reference.clone());
        }
    }
    references
}

fn init_tracing(
    otel_endpoint: Option<&str>,
    otel_service_name: &str,
    otel_sampling_percentage: f64,
    otel_tags: Option<&str>,
) -> std::io::Result<bool> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer();
    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if let Some(endpoint) = otel_endpoint {
        let sampling_percentage = if otel_sampling_percentage.is_finite() {
            otel_sampling_percentage.clamp(0.0, 100.0)
        } else {
            100.0
        };
        let sampling_ratio = sampling_percentage / 100.0;
        let mut resource_attributes =
            vec![KeyValue::new("service.name", otel_service_name.to_string())];
        resource_attributes.extend(parse_otel_tags(otel_tags)?);
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint.to_string()),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::config()
                    .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                        sampling_ratio,
                    ))))
                    .with_resource(Resource::new(resource_attributes)),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .map_err(|err| std::io::Error::other(format!("initialize OTEL tracing: {err}")))?;
        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
        info!(
            otel_endpoint = %endpoint,
            otel_service_name = %otel_service_name,
            otel_sampling_percentage = sampling_percentage,
            "OpenTelemetry tracing enabled"
        );
        Ok(true)
    } else {
        registry.init();
        Ok(false)
    }
}

fn parse_otel_tags(raw: Option<&str>) -> std::io::Result<Vec<KeyValue>> {
    let Some(raw) = raw.filter(|value| !value.trim().is_empty()) else {
        return Ok(Vec::new());
    };
    let tags: std::collections::BTreeMap<String, String> = serde_json::from_str(raw)
        .map_err(|err| std::io::Error::other(format!("parse TRANSIT_OTEL_TAGS: {err}")))?;
    Ok(tags
        .into_iter()
        .map(|(name, value)| KeyValue::new(name, value))
        .collect())
}

fn apply_bootstrap(args: &mut Args, bootstrap: BootstrapConfig) {
    if let Some(value) = bootstrap.xds_address {
        args.xds_address = value;
    }
    if let Some(value) = bootstrap.http_addr {
        args.http_addr = value;
    }
    if let Some(value) = bootstrap.ui_addr {
        args.ui_addr = value;
    }
    if !bootstrap.listener_names.is_empty() {
        args.listener_names = bootstrap.listener_names;
    }
    if let Some(value) = bootstrap.pod_name {
        args.pod_name = value;
    }
    if let Some(value) = bootstrap.namespace {
        args.namespace = value;
    }
    if let Some(value) = bootstrap.pod_ip {
        args.pod_ip = value;
    }
    if let Some(value) = bootstrap.node_name {
        args.node_name = Some(value);
    }
    if let Some(value) = bootstrap.cluster_id {
        args.cluster_id = value;
    }
    if let Some(value) = bootstrap.dns_domain {
        args.dns_domain = value;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_api_and_management_ports_match_the_shared_contract() {
        use clap::CommandFactory;
        let command = super::Args::command();
        for (name, address) in [("http_addr", "0.0.0.0:26080"), ("ui_addr", "0.0.0.0:26021")] {
            let arg = command
                .get_arguments()
                .find(|arg| arg.get_id() == name)
                .unwrap();
            assert_eq!(
                arg.get_default_values(),
                &[std::ffi::OsString::from(address)]
            );
        }
        assert_eq!(transit_core::HTTPS_LISTENER_PORT, 26443);
    }

    #[test]
    fn loopback_management_initializes_without_env_and_remote_requires_token() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("transit-local-init-{nonce}"));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let local = super::ProxyState::new();
        super::configure_llm_accounts(
            &local,
            "127.0.0.1:26021".parse().unwrap(),
            Some(&directory),
            None,
        )
        .unwrap();
        assert!(local.llm_accounts().enabled());
        let token = local.llm_accounts().local_session_token().unwrap();
        assert!(local.llm_accounts().authorized(&token));
        let restarted = super::ProxyState::new();
        super::configure_llm_accounts(
            &restarted,
            "127.0.0.1:26021".parse().unwrap(),
            Some(&directory),
            None,
        )
        .unwrap();
        assert!(!restarted.llm_accounts().authorized(&token));

        let remote = super::ProxyState::new();
        let err = super::configure_llm_accounts(
            &remote,
            "0.0.0.0:26021".parse().unwrap(),
            Some(&directory),
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("TRANSIT_LLM_ADMIN_TOKEN"));
        assert!(!remote.llm_accounts().enabled());
        super::configure_llm_accounts(
            &remote,
            "0.0.0.0:26021".parse().unwrap(),
            Some(&directory),
            Some("test-explicit-management-token-long-enough"),
        )
        .unwrap();
        assert!(remote.llm_accounts().enabled());
        assert!(remote
            .llm_accounts()
            .authorized("test-explicit-management-token-long-enough"));
        assert!(remote.llm_accounts().local_session_token().is_none());
    }
    use super::{apply_bootstrap, parse_otel_tags, Args};
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use transit_xds::BootstrapConfig;

    fn base_args() -> Args {
        Args {
            mode: transit_core::RuntimeMode::Standalone,
            xds_address: "http://old:26012".to_string(),
            xds_enabled: None,
            gateway: None,
            xds_root_ca: None,
            xds_token_file: None,
            http_addr: "0.0.0.0:26080".parse().unwrap(),
            ui_addr: "0.0.0.0:26021".parse().unwrap(),
            llm_accounts_dir: None,
            llm_admin_token: None,
            access_config: None,
            metrics_enabled: true,
            drain_timeout_seconds: 30,
            bootstrap: Some(PathBuf::from("/etc/transit/bootstrap.json")),
            static_config: None,
            otel_endpoint: None,
            otel_service_name: "transit".to_string(),
            otel_sampling_percentage: 100.0,
            otel_tags: None,
            listener_names: Vec::new(),
            pod_name: "transit".to_string(),
            namespace: "dubbo-system".to_string(),
            pod_ip: "127.0.0.1".to_string(),
            node_name: None,
            cluster_id: "old-cluster".to_string(),
            dns_domain: "cluster.local".to_string(),
            command: None,
        }
    }

    #[test]
    fn parses_otel_tags_json() {
        let tags = parse_otel_tags(Some(r#"{"foo":"bar","userId":"unknown"}"#)).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].key.as_str(), "foo");
        assert_eq!(tags[0].value.as_str(), "bar");
        assert_eq!(tags[1].key.as_str(), "userId");
        assert_eq!(tags[1].value.as_str(), "unknown");
    }

    #[test]
    fn bootstrap_overrides_control_plane_fields() {
        let mut args = base_args();

        apply_bootstrap(
            &mut args,
            BootstrapConfig {
                xds_address: Some("http://dubbod.dubbo-system.svc:26012".to_string()),
                http_addr: Some("0.0.0.0:8080".parse::<SocketAddr>().unwrap()),
                listener_names: vec!["public-dubbo.app.svc.cluster.local:80".to_string()],
                cluster_id: Some("Kubernetes".to_string()),
                dns_domain: Some("svc.local".to_string()),
                ..BootstrapConfig::default()
            },
        );

        assert_eq!(args.xds_address, "http://dubbod.dubbo-system.svc:26012");
        assert_eq!(args.http_addr.port(), 8080);
        assert_eq!(args.cluster_id, "Kubernetes");
        assert_eq!(args.dns_domain, "svc.local");
        assert_eq!(args.pod_name, "transit");
        assert_eq!(
            args.listener_names,
            ["public-dubbo.app.svc.cluster.local:80"]
        );
    }

    #[test]
    fn demo_config_deserializes() {
        let raw = include_str!("../../../tests/ui-fake.json");
        let cfg: transit_core::RuntimeConfig = serde_json::from_str(raw).unwrap();
        assert_eq!(cfg.clusters.len(), 3);
        assert_eq!(cfg.providers.len(), 2);
        assert_eq!(cfg.backends.len(), 7);
    }
}
