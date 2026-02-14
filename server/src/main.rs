mod cli;
mod http;
mod rpc;

use crate::cli::{Cli, Parser};
use crate::http::HttpServer;
use crate::rpc::RpcServer;

pub use kid_app::server::ssr::SharedTaskCache;
use kid_types::server::FlushError;
use kid_types::server::TaskCache;

use anyhow::Result;
use leptos::prelude::get_configuration;
use miette::MietteHandlerOpts;
use tokio::signal;
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::time::{self, Duration, Instant};
use tokio_util::sync::CancellationToken;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    init_miette_report();
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();
    let leptos_conf = get_configuration(None)?;
    let leptos_options = leptos_conf.leptos_options.clone();
    let site_addr = leptos_conf.leptos_options.site_addr;
    let rpc_addr = args.server.addr;

    let rpc_listener = tokio::net::TcpListener::bind(&rpc_addr).await?;
    let web_listener = tokio::net::TcpListener::bind(&site_addr).await?;

    let task_cache: SharedTaskCache = Arc::new(RwLock::new(TaskCache::default()));

    let shutdown = CancellationToken::new();
    let background_flush = {
        let shutdown = shutdown.clone();
        let task_cache = task_cache.clone();
        spawn(async move {
            task_cache.background_flush(shutdown).await;
        })
    };
    let rpc = spawn(RpcServer::serve(
        rpc_listener,
        shutdown.clone(),
        task_cache.clone(),
    ));
    let http = spawn(HttpServer::serve(
        web_listener,
        leptos_options,
        shutdown.clone(),
        task_cache.clone(),
    ));
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C signal handler");
        tracing::info!("signal received: Ctrl+C");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install terminate signal handler")
            .recv()
            .await;
        tracing::info!("signal received: terminate");
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    shutdown.cancel();
    let _ = tokio::try_join!(background_flush, rpc, http);

    tracing::warn!("Gracefully shutdown started..");
    task_cache.final_flush().await;
    tracing::info!("Bye bye");
    Ok(())
}

trait TaskCacheFlush<'a> {
    const FLUSH_INTERVAL: Duration;
    const FLUSH_TIMEOUT: Duration;

    async fn background_flush(&self, shutdown: CancellationToken);
    async fn final_flush(&self);
}

impl<'a> TaskCacheFlush<'a> for SharedTaskCache {
    const FLUSH_INTERVAL: Duration = Duration::from_mins(1);
    const FLUSH_TIMEOUT: Duration = Duration::from_secs(4);

    async fn background_flush(&self, shutdown: CancellationToken) {
        let mut interval = time::interval(Self::FLUSH_INTERVAL);
        let mut flush_failed_count = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    tracing::trace!("flush task cache..");
                    let mut cache = self.write().await;
                    match cache.flush().await {
                        Ok(num) => {
                            if num > 0 {
                                tracing::debug!("flush task cache.. {num} tasks successfully flushed.");
                            } else {
                                tracing::trace!("flush task cache.. done.");
                            }
                            flush_failed_count = 0;
                        }
                        Err(e) => {
                            tracing::warn!("flush task cache.. with errors: {e}");
                            match e {
                                FlushError::ErrorList(failed, _, _) => {
                                    interval.reset_after(Self::FLUSH_INTERVAL / (failed + 1) as u32);
                                }
                                _ => {
                                    interval.reset_after(Self::FLUSH_INTERVAL / 2);
                                }
                            }

                            flush_failed_count += 1;
                            if flush_failed_count > 10 {
                                // reset to prevent error spamming
                                flush_failed_count = 0;
                                // we want a detailed error report (with suberrors)
                                let e = miette::Report::from(e);
                                tracing::error!("failed to flush task cache:\n{e:?}")
                            }
                        }
                    }
                }
                _ = shutdown.cancelled() => {
                    tracing::info!("Background task flush shutting down");
                    break;
                }
            }
        }
    }

    async fn final_flush(&self) {
        tracing::trace!("flush task cache finally..");
        let mut last_error: Option<FlushError> = None;
        let start = Instant::now();
        let mut cache = self.write().await;
        while start.elapsed() < Self::FLUSH_TIMEOUT {
            match cache.flush().await {
                Ok(num) => {
                    tracing::info!("flush task cache.. {num} tasks successfully flushed.");
                    return;
                }
                Err(e) => {
                    tracing::warn!("flush task cache.. with errors: {e}");
                    last_error = Some(e);
                    // give storage IO some time to relax..
                    time::sleep(Duration::from_millis(10)).await;
                }
            }
        }

        if let Some(e) = last_error {
            // we want a detailed error report (with suberrors)
            let e = miette::Report::from(e);
            tracing::error!(
                "flush task cache.. timeout after {:?} with error:\n{e:?}",
                Self::FLUSH_TIMEOUT,
            )
        } else {
            tracing::info!("flush task cache.. successfully completed");
        }
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
