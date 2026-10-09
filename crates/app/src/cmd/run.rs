use crate::{RunArgs, read_config_contents};

use gateway::{Config, serdes};
use std::sync::Arc;
use tracing::info;
use transit_core::telemetry;
pub(crate) fn execute(args: RunArgs) -> anyhow::Result<()> {
    let RunArgs { config } = args;

    let (contents, local_config_source) = read_config_contents(&config)?;
    let config = gateway::config::parse_config(contents, local_config_source)?;
    let worker_threads = config.num_worker_threads;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()?;
    telemetry::setup_logging();
    runtime.block_on(proxy(Arc::new(config)))
}

async fn proxy(cfg: Arc<Config>) -> anyhow::Result<()> {
    info!(
        "Using Configuration: {}",
        serdes::yamlviajson::to_string(cfg.as_ref())?
    );
    gateway::app::wait_termination(cfg.termination_min_deadline, cfg.termination_max_deadline).await
}
