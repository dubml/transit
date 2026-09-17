use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{info, warn};
use transit::RuntimeConfig;
use transit_proxy::{ProxyServer, ProxyState};

#[derive(Debug, Parser)]
#[command(name = "transit", version, about = "Transit AI & API Gateway")]
pub struct Args {
    /// Path to configuration file (YAML or JSON).
    #[arg(short, long, env = "TRANSIT_CONFIG")]
    pub config: Option<PathBuf>,

    /// Validate configuration and exit without starting server.
    #[arg(long)]
    pub validate: bool,
}

pub async fn run() -> Result<()> {
    init_logging();
    let args = Args::parse();

    let config_path = args.config.unwrap_or_else(|| PathBuf::from("config.yaml"));
    let config = if config_path.exists() {
        load_config(&config_path)?
    } else {
        warn!(path = ?config_path, "Configuration file not found, starting with empty configuration");
        RuntimeConfig::empty("default")
    };

    if args.validate {
        info!("Configuration validation succeeded");
        return Ok(());
    }

    let state = ProxyState::new();
    if let Err(conflicts) = state.apply_config(config) {
        for conflict in conflicts {
            warn!(conflict = ?conflict, "Configuration conflict detected on bootstrap");
        }
    }

    let server = ProxyServer::new(state);
    info!("Starting Transit Gateway...");

    let shutdown_signal = async {
        let _ = tokio::signal::ctrl_c().await;
        info!("Shutdown signal received, draining connections...");
    };

    server
        .serve_configured_with_shutdown(Duration::from_secs(10), shutdown_signal)
        .await
        .context("Proxy server failed")?;

    info!("Transit Gateway stopped");
    Ok(())
}

fn load_config(path: &Path) -> Result<RuntimeConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;

    if path.extension().is_some_and(|ext| ext == "json") {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON configuration: {}", path.display()))
    } else {
        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML configuration: {}", path.display()))
    }
}

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}
