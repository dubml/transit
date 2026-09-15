use super::*;

pub(crate) async fn run(args: Args) -> std::io::Result<()> {
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

    run_proxy(args).await
}

async fn run_proxy(mut args: Args) -> std::io::Result<()> {
    let _ = rustls_kube::crypto::ring::default_provider().install_default();

    if let Some(path) = args.bootstrap.clone() {
        let bootstrap = BootstrapConfig::load(path)
            .await
            .map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
            })?;
        apply_bootstrap(&mut args, bootstrap);
    }

    // 把原 main() 从 init_tracing 一直到 Ok(()) 的内容原样放这里。
    //
    // 暂时不要继续抽 Config / Server / Shutdown。
    // 第一刀只建立：
    //
    // main -> run::run -> run_proxy
    //
    // 编译通过后再拆下一层。

    Ok(())
}