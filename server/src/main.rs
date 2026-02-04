mod cli;
mod http;
mod rpc;

use crate::cli::{Cli, Parser};
use crate::http::HttpServer;
use crate::rpc::RpcServer;

pub use kid_app::server::ssr::SharedTaskCache;
use kid_types::server::TaskCache;

use anyhow::Result;
use leptos::prelude::get_configuration;
use tokio::sync::RwLock;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
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

    let rpc = RpcServer::serve(rpc_listener, task_cache.clone());
    let http = HttpServer::serve(web_listener, leptos_options, task_cache);
    tokio::try_join!(rpc, http)?;

    Ok(())
}
