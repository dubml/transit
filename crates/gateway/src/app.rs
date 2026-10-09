use std::time::Duration;

pub async fn wait_termination(
    termination_min_deadline: Duration,
    termination_max_deadline: Duration,
) -> anyhow::Result<()> {
    wait_for_signal().await?;

    tokio::select! {
        _ = tokio::time::sleep(termination_min_deadline) => {},
        _ = tokio::time::sleep(termination_max_deadline) => {},
    }
    Ok(())
}

#[cfg(unix)]
async fn wait_for_signal() -> anyhow::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => result?,
        _ = terminate.recv() => {},
    }
    Ok(())
}

#[cfg(not(unix))]
async fn wait_for_signal() -> anyhow::Result<()> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}
