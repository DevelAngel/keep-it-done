mod cli;
mod task;

use crate::cli::{Cli, Parser};
use crate::task::TaskPrint;

use kid_types::Task;
use kid_types::rpc::TaskServiceClient;

use anyhow::{Context, Result};
use schemars::SchemaGenerator;
use tarpc::client;
use tarpc::context;
use tarpc::serde_transport::tcp;
use tarpc::tokio_serde::formats::Json;
use tracing::Instrument;

use std::fs::File;
use std::io::{self, BufWriter};
use std::net::SocketAddr;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    match args.cmd {
        cli::Commands::Schema { pretty, outfile } => schema(pretty, outfile.as_deref()).await?,
        cli::Commands::List { server } => {
            let client = connect(&server.addr).await?;
            list(client).await?;
        }
        cli::Commands::Add { server, summary } => {
            let client = connect(&server.addr).await?;
            add(client, summary).await?;
        }
    }
    Ok(())
}

async fn schema(pretty: bool, outfile: Option<&Path>) -> Result<()> {
    let generator = SchemaGenerator::default();
    let schema = generator.into_root_schema_for::<Task>();
    if let Some(outfile) = outfile {
        let file = File::create(outfile)?;
        let writer = BufWriter::new(file);
        if pretty {
            serde_json::to_writer_pretty(writer, &schema)?;
        } else {
            serde_json::to_writer(writer, &schema)?;
        }
    } else {
        let stdout = io::stdout().lock();
        let writer = BufWriter::new(stdout);
        if pretty {
            serde_json::to_writer_pretty(writer, &schema)?;
        } else {
            serde_json::to_writer(writer, &schema)?;
        }
    };
    Ok(())
}

async fn list(client: TaskServiceClient) -> Result<()> {
    let task_list = async move {
        // Send the request twice, just to be safe! ;)
        tokio::select! {
            task_list_one = client.list(context::current()) => { task_list_one }
            task_list_two = client.list(context::current()) => { task_list_two }
        }
    }
    .instrument(tracing::info_span!("Two Task Lists"))
    .await
    .context("failed to fetch the task list")?;

    // Print task list to standard out
    task_list
        .iter()
        .map(TaskPrint)
        .enumerate()
        .map(|(i, task)| (i + 1, task))
        .for_each(|(n, task)| println!("{n:>3}: {task}"));

    Ok(())
}

async fn add(client: TaskServiceClient, summary: String) -> Result<()> {
    let task = Task::new(summary);
    client
        .add(context::current(), task)
        .await
        .context("failed to add the new task")?;
    Ok(())
}

async fn connect(addr: &SocketAddr) -> Result<TaskServiceClient> {
    tracing::info!("CLI will connect to {}", addr);
    let mut transport = tcp::connect(addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let transport = transport.await.context("failed to connect")?;
    let client = TaskServiceClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}
