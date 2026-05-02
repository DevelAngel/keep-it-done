mod cli;

use kid_server::builder::ServerBuilder;
use kid_server::cache::graceful_shutdown;
use kid_server::SharedTaskCache;
use crate::cli::{Cli, Parser};

use leptos::prelude::get_configuration;
use miette::Result;
use miette::{IntoDiagnostic, MietteHandlerOpts};
use tokio::signal;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    init_miette_report();
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    let leptos_conf = get_configuration(None).into_diagnostic()?;
    let http_addr = leptos_conf.leptos_options.site_addr;
    let rpc_addr = args.server.addr;

    let task_cache = SharedTaskCache::default();
    task_cache.write().await.load().await.and_then(|(num_loaded, num_to_migrate)| {
        if num_loaded > 0 {
            tracing::info!("{num_loaded} tasks loaded");
            if num_to_migrate > 0 {
                tracing::info!("{num_to_migrate} tasks has to be migrated with next flush");
            }
        } else {
            tracing::warn!("no tasks loaded");
        }
        Ok(())
    })?;

    let shutdown = CancellationToken::new();
    let server = ServerBuilder::new(&shutdown, &task_cache)
        .with_rpc_addr(&rpc_addr)
        .with_http_addr(&http_addr)
        .with_leptos_options(&leptos_conf.leptos_options)
        .try_spawn()
        .await?;

    setup_signals().await;
    graceful_shutdown(shutdown, task_cache).await;
    server.join().await?;
    Ok(())
}

async fn setup_signals() {
    use signal::ctrl_c;
    use signal::unix::{SignalKind, signal};
    tokio::select! {
        _ = ctrl_c() => tracing::info!("signal received: Ctrl+C"),
        _ = async {
            signal(SignalKind::terminate())
                .expect("failed to install terminate signal handler")
                .recv()
                .await
        } => tracing::info!("signal received: terminate"),
    }
}

fn init_miette_report() {
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .color(false)
                .without_syntax_highlighting()
                .terminal_links(true)
                .unicode(true)
                .build(),
        )
    }));
}
