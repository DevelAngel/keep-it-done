mod cli;
mod rpc;

use crate::cli::{Cli, Parser};
use crate::rpc::{TaskRpcServer, TaskService};

use futures::{future, prelude::*};
use tarpc::serde_transport::tcp;
use tarpc::server::incoming::Incoming;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();
    tracing::warn!("work-in-progress");
    tracing::error!("test");
    tracing::debug!("test");
    tracing::trace!("test");
    tracing::info!("SERVER will listen to {}", args.server.addr);

    // Start RPC service
    let mut listener = tcp::listen(&args.server.addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = TaskRpcServer;
            channel.execute(server.serve()).for_each(spawn)
        })
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
