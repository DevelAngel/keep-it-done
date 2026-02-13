mod cli;
mod http;
mod rpc;

use crate::cli::{Cli, Parser};
use crate::http::HttpServer;
use crate::rpc::RpcServer;

pub use kid_app::server::ssr::SharedTaskCache;
use kid_types::server::{FlushError, TaskCache};

use anyhow::Result;
use leptos::prelude::get_configuration;
use miette::MietteHandlerOpts;
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};

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

    tracing::debug!("prepare task cache..");
    let task_cache: SharedTaskCache = Arc::new(RwLock::new(TaskCache::default()));
    let backup_handle = SharedTaskCache::backup_spawn(task_cache.clone(), Duration::from_mins(1));
    tracing::debug!("prepare task cache.. done.");

    let rpc = RpcServer::serve(rpc_listener, task_cache.clone());
    let http = HttpServer::serve(web_listener, leptos_options, task_cache);

    tokio::join!(backup_handle);
    tokio::try_join!(rpc, http)?;

    Ok(())
}

trait CacheBackup {
    async fn backup_spawn(cache: SharedTaskCache, interval: Duration) -> JoinHandle<()>;
}

impl CacheBackup for SharedTaskCache {
    async fn backup_spawn(cache: SharedTaskCache, interval: Duration) -> JoinHandle<()> {
        spawn(async move {
            let max_interval = interval;
            let mut interval = time::interval(max_interval);
            let mut flush_failed_count = 0;
            loop {
                interval.tick().await;

                tracing::trace!("flush task cache..");
                let mut cache = cache.write().await;
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
                                interval.reset_after(max_interval / (failed + 1) as u32);
                            }
                            _ => {
                                interval.reset_after(max_interval / 2);
                            }
                        }

                        flush_failed_count += 1;
                        if flush_failed_count > 10 {
                            // reset to prevent error spamming
                            flush_failed_count = 0;
                            // we want a detailed error report (with suberrors)
                            let e = miette::Report::from(e);
                            tracing::error!("failed to flush task cache: {e:?}")
                        }
                    }
                }
            }
        })
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
