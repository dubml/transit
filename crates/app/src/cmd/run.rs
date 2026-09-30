use crate::{RunArgs, read_config_contents};

use gateway::{Config, serdes};
use std::sync::Arc;
use tracing::info;
pub(crate) fn execute(args: RunArgs) -> anyhow::Result<()> {
    let RunArgs { config } = args;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let (contents, local_config_source) = read_config_contents(&config)?;
            let config = gateway::config::parse_config(contents, local_config_source)?;
            let result = proxy(Arc::new(config)).await;
            result
        })
}

async fn proxy(cfg: Arc<Config>) -> anyhow::Result<()> {
    info!(
        "Using Configuration: {}",
        serdes::yamlviajson::to_string(cfg.as_ref())?
    );
    Ok(())
}
