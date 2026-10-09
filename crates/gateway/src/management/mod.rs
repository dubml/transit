mod admin;
mod drain;
mod health;
mod hyper_helpers;
mod readiness;

use crate::Address;
use readiness::Ready;

pub use drain::{DrainSignal as Drain, DrainWatcher};

pub async fn start(
    admin_addr: Address,
    health_addr: Address,
    config: String,
    ready: Ready,
    drain: DrainWatcher,
) -> anyhow::Result<Vec<tokio::task::JoinHandle<()>>> {
    let admin = admin::Server::new(admin_addr, config, drain.clone()).await?;
    let health = health::Server::new(health_addr, ready, drain).await?;
    Ok(vec![admin.spawn(), health.spawn()])
}

pub use readiness::Ready as Readiness;
pub use readiness::Task as ReadinessTask;
