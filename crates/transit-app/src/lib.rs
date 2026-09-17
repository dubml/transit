mod stealth;

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde_json::json;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};
use transit::{ProxyServer, RuntimeConfig};

#[derive(Debug, Parser)]
#[command(name = "transit", about = "Transit AI & API Gateway", disable_version_flag = true)]
pub struct Cli {
    #[command(flatten)]
    pub run: RunArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, ClapArgs)]
pub struct RunArgs {
    /// Path to configuration file (use '-' for standard input).
    #[arg(short = 'f', long = "file", value_name = "FILE", env = "TRANSIT_CONFIG")]
    pub file: Option<PathBuf>,

    /// Path to configuration file (alias for --file).
    #[arg(short = 'c', long = "config", value_name = "CONFIG")]
    pub config: Option<PathBuf>,

    /// Validate configuration and exit without starting server.
    #[arg(long)]
    pub validate: bool,

    /// Print version information in JSON format.
    #[arg(short = 'V', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run transit in background as a transient process, execute command when ready, then clean up.
    Stealth(stealth::StealthArgs),
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Stealth(args)) = cli.command {
        return stealth::execute(args);
    }

    if cli.run.version {
        print_version_json();
        return Ok(());
    }

    init_logging();

    let config_path = cli
        .run
        .file
        .as_ref()
        .or(cli.run.config.as_ref());

    let (config, content_desc) = match config_path {
        Some(path) if path == Path::new("-") => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read configuration from standard input")?;
            let cfg = parse_config_str(&buffer, "stdin")?;
            (cfg, "standard input".to_string())
        }
        Some(path) => {
            let cfg = load_config_file(path)?;
            (cfg, path.display().to_string())
        }
        None => {
            let default_path = PathBuf::from("config.yaml");
            if default_path.exists() {
                let cfg = load_config_file(&default_path)?;
                (cfg, default_path.display().to_string())
            } else {
                warn!(path = ?default_path, "Configuration file not found, starting with empty configuration");
                (RuntimeConfig::empty("default"), "empty default".to_string())
            }
        }
    };

    if cli.run.validate {
        info!(source = %content_desc, "Configuration validation succeeded");
        return Ok(());
    }

    if let Err(conflicts) = config.validate() {
        for conflict in conflicts {
            warn!(conflict = ?conflict, "Configuration conflict detected on bootstrap");
        }
    }

    let config = Arc::new(config);
    notify_readiness_if_configured(&config);

    let server = ProxyServer::new(config.clone());
    info!(source = %content_desc, "Starting Transit Gateway...");

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

fn print_version_json() {
    let version_info = json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "license": "Apache-2.0",
        "repository": env!("CARGO_PKG_REPOSITORY"),
    });
    println!("{}", serde_json::to_string_pretty(&version_info).unwrap());
}

fn load_config_file(path: &Path) -> Result<RuntimeConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;
    parse_config_str(&content, &path.display().to_string())
}

fn parse_config_str(content: &str, source_name: &str) -> Result<RuntimeConfig> {
    if let Ok(cfg) = serde_json::from_str::<RuntimeConfig>(content) {
        return Ok(cfg);
    }
    serde_yaml::from_str::<RuntimeConfig>(content)
        .with_context(|| format!("Failed to parse configuration from {source_name} as YAML or JSON"))
}

fn notify_readiness_if_configured(config: &Arc<RuntimeConfig>) {
    #[cfg(unix)]
    {
        if let Ok(val) = std::env::var("TRANSIT_READY_FD") {
            if let Ok(fd) = val.parse::<i32>() {
                let _ = config;
                tokio::spawn(async move {
                    // Small delay to allow listeners to bind on first iteration
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    unsafe {
                        let byte = [1u8];
                        let res = libc::write(fd, byte.as_ptr() as *const libc::c_void, 1);
                        if res == -1 {
                            let err = std::io::Error::last_os_error();
                            if err.kind() != std::io::ErrorKind::BrokenPipe {
                                tracing::debug!(%err, "readiness notification pipe write returned error");
                            }
                        }
                        libc::close(fd);
                    }
                });
            }
        }
    }
    #[cfg(not(unix))]
    let _ = config;
}

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}
