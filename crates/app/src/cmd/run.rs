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
    let config_dump = serdes::yamlviajson::to_string(cfg.as_ref())?;
    info!("Using Configuration: |-\n{}", config_dump);
    let ready = gateway::management::Readiness::default();
    let state_manager_task = cfg
        .xds
        .local_config
        .as_ref()
        .map(|_| ready.register_task("state manager"));
    let app_task = ready.register_task("transit");
    let (drain, drain_watcher) = gateway::management::Drain::new();
    let management_tasks = gateway::app::start_management(
        cfg.admin_addr.clone(),
        cfg.health_addr.clone(),
        config_dump,
        ready,
        drain_watcher,
    )
    .await?;
    let state_manager = match cfg.xds.local_config.clone() {
        Some(source) => Some(gateway::state_manager::StateManager::new(source).await?),
        None => None,
    };
    let state_task = state_manager.map(|manager| tokio::spawn(manager.run()));
    drop(state_manager_task);
    drop(app_task);
    let result = gateway::app::wait_termination(
        cfg.termination_min_deadline,
        cfg.termination_max_deadline,
        drain,
        management_tasks,
    )
    .await;
    if let Some(task) = state_task {
        task.abort();
    }
    result
}
