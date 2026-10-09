use std::time::{Duration, Instant};
use tracing::info;

pub async fn start_management(
    admin_addr: crate::Address,
    health_addr: crate::Address,
    config: String,
    ready: crate::management::Readiness,
    drain: crate::management::DrainWatcher,
) -> anyhow::Result<Vec<tokio::task::JoinHandle<()>>> {
    crate::management::start(admin_addr, health_addr, config, ready, drain).await
}

pub async fn wait_termination(
    termination_min_deadline: Duration,
    termination_max_deadline: Duration,
    drain: crate::management::Drain,
    mut management_tasks: Vec<tokio::task::JoinHandle<()>>,
) -> anyhow::Result<()> {
    let signal = wait_for_signal().await?;
    info!(%signal, "Shutdown signal received; draining listeners");
    drain.signal();

    let started = Instant::now();
    let _ = tokio::time::timeout(termination_max_deadline, async {
        for task in &mut management_tasks {
            let _ = task.await;
        }
    })
    .await;
    if let Some(remaining) = termination_min_deadline.checked_sub(started.elapsed()) {
        tokio::time::sleep(remaining).await;
    }
    for task in management_tasks {
        if !task.is_finished() {
            task.abort();
        }
    }
    Ok(())
}

#[cfg(unix)]
async fn wait_for_signal() -> anyhow::Result<&'static str> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => { result?; Ok("SIGINT") },
        _ = terminate.recv() => Ok("SIGTERM"),
    }
}

#[cfg(not(unix))]
async fn wait_for_signal() -> anyhow::Result<&'static str> {
    tokio::signal::ctrl_c().await?;
    Ok("SIGINT")
}
