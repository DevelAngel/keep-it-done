mod cli;
mod task;

use crate::cli::{Cli, Parser};
use crate::task::TaskPrint;

use kid_types::TaskServiceClient;

use anyhow::{Context, Result};
use tarpc::client;
use tarpc::context;
use tarpc::serde_transport::tcp;
use tarpc::tokio_serde::formats::Json;
use tracing::Instrument;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();
    tracing::warn!("work-in-progress");
    tracing::error!("test");
    tracing::debug!("test");
    tracing::trace!("test");
    tracing::info!("CLI will connect to {}", args.server.addr);

    // Connect to server and fetch task list
    let mut transport = tcp::connect(args.server.addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let transport = transport.await.context("failed to connect")?;
    let client = TaskServiceClient::new(client::Config::default(), transport).spawn();
    let task_list = async move {
        // Send the request twice, just to be safe! ;)
        tokio::select! {
            task_list_one = client.list(context::current()) => { task_list_one }
            task_list_two = client.list(context::current()) => { task_list_two }
        }
    }
    .instrument(tracing::info_span!("Two Task Lists"))
    .await
    .context("Error message received")?;

    // Print task list to standard out
    task_list
        .iter()
        .map(TaskPrint)
        .enumerate()
        .map(|(i, task)| (i + 1, task))
        .for_each(|(n, task)| println!("{n:>3}: {task}"));
    Ok(())
}
