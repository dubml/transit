mod stealth;

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};
use transit::{ProxyServer, RuntimeConfig, StateManager};

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

    info!(
        version = env!("CARGO_PKG_VERSION"),
        target = %format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH),
        "Transit Gateway"
    );

    let config_path = cli
        .run
        .file
        .as_ref()
        .or(cli.run.config.as_ref());

    let start_time = std::time::Instant::now();
    let (state_manager, content_desc) = StateManager::init(config_path.map(|p| p.as_path()))?;
    let config = state_manager.current_config();

    if cli.run.validate {
        info!(source = %content_desc, "Configuration validation succeeded");
        return Ok(());
    }

    if let Err(conflicts) = config.validate() {
        for conflict in conflicts {
            warn!(conflict = ?conflict, "Configuration conflict detected on bootstrap");
        }
    }

    notify_readiness_if_configured(&config);

    let server = ProxyServer::new(config.clone());
    let _watcher = state_manager.start_watcher(server.clone());

    let duration = start_time.elapsed();
    info!(target: "readiness", "Task 'state manager' complete ({:?}), still awaiting 1 tasks", duration);

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
    let target = format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH);
    let version_info = json!({
        "target": target,
        "version": env!("CARGO_PKG_VERSION"),
    });
    println!("{}", serde_json::to_string_pretty(&version_info).unwrap());
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
    let base_filter = match std::env::var("RUST_LOG") {
        Ok(ref val) if !val.trim().is_empty() => {
            format!("{val},state_manager=info,readiness=info,app=info,transit=info,transit_app=info")
        }
        _ => "info".to_string(),
    };

    let filter = tracing_subscriber::EnvFilter::try_new(&base_filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init();
}
