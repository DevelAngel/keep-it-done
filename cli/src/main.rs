mod cli;
mod task;

use crate::cli::{Cli, Commands, Parser, ServerArgs};
use crate::task::{TaskDetails, TaskDetailsPatch, TaskPrint};

use kid_types::rpc::TaskServiceClient;
use kid_types::{Task, TaskCategory, TaskSummary, Uuid};

use miette::{IntoDiagnostic, Result, WrapErr};
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
    init_miette_report();
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    match args.cmd {
        Commands::Schema { pretty, outfile } => schema(pretty, outfile.as_deref()).await?,
        Commands::List {
            server,
            json,
            pretty,
        } => {
            list(&server, json, pretty).await?;
        }
        Commands::Add {
            server,
            summary,
            category,
            details,
        } => {
            add(&server, summary, category, details.as_deref()).await?;
        }
        Commands::Rename {
            server,
            id,
            summary,
        } => {
            rename(&server, &id, summary).await?;
        }
        Commands::Replace {
            server,
            id,
            details,
        } => {
            replace(&server, &id, details).await?;
        }
        Commands::Update {
            server,
            id,
            details,
        } => {
            update(&server, &id, details).await?;
        }
        Commands::Complete { server, id, reopen } => {
            complete(&server, &id, reopen).await?;
        }
    }
    Ok(())
}

async fn schema(pretty: bool, outfile: Option<&Path>) -> Result<()> {
    use crate::task::Details as TaskDetails;
    let generator = SchemaGenerator::default();
    let schema = generator.into_root_schema_for::<TaskDetails>();
    if let Some(outfile) = outfile {
        let file = File::create(outfile).into_diagnostic()?;
        let writer = BufWriter::new(file);
        if pretty {
            serde_json::to_writer_pretty(writer, &schema).into_diagnostic()?;
        } else {
            serde_json::to_writer(writer, &schema).into_diagnostic()?;
        }
    } else {
        let stdout = io::stdout().lock();
        let writer = BufWriter::new(stdout);
        if pretty {
            serde_json::to_writer_pretty(writer, &schema).into_diagnostic()?;
        } else {
            serde_json::to_writer(writer, &schema).into_diagnostic()?;
        }
    };
    Ok(())
}

async fn list(server: &ServerArgs, json: bool, pretty: bool) -> Result<()> {
    let client = connect(&server.addr).await?;
    let task_list = async move {
        // Send the request twice, just to be safe! ;)
        tokio::select! {
            task_list_one = client.list(context::current()) => { task_list_one }
            task_list_two = client.list(context::current()) => { task_list_two }
        }
    }
    .instrument(tracing::info_span!("Two Task Lists"))
    .await
    .into_diagnostic()
    .wrap_err("failed to fetch the task list")?;

    // Print task list to standard out
    if json {
        let task_list: Vec<_> = task_list
            .iter()
            .map(|(id, task)| TaskPrint::new(id, task))
            .collect();
        let stdout = io::stdout().lock();
        let writer = BufWriter::new(stdout);
        if pretty {
            serde_json::to_writer_pretty(writer, &task_list).into_diagnostic()?;
        } else {
            serde_json::to_writer(writer, &task_list).into_diagnostic()?;
        }
    } else {
        task_list
            .iter()
            .map(|(id, task)| TaskPrint::new(id, task))
            .enumerate()
            .map(|(i, task)| (i + 1, task))
            .for_each(|(n, task)| println!("{n:>3}: {task}"));
    }

    Ok(())
}

async fn add(server: &ServerArgs, summary: String, category: String, details: Option<&str>) -> Result<()> {
    let summary: TaskSummary = summary.parse().map_err(|e| miette::miette!("{e}"))?;
    let category: TaskCategory = category.parse().map_err(|e| miette::miette!("{e}"))?;
    let mut task = Task::new(summary).with_category(category);
    if let Some(details) = details {
        let details: TaskDetails = details.parse()?;
        task = task.with_details(details.into());
    }
    tracing::debug!("task created:\n{task:#?}");

    let client = connect(&server.addr).await?;
    client
        .add(context::current(), task)
        .await
        .into_diagnostic()
        .wrap_err("failed to add task")?;
    Ok(())
}

async fn rename(server: &ServerArgs, id: &Uuid, summary: String) -> Result<()> {
    let summary: TaskSummary = summary.parse().map_err(|e| miette::miette!("{e}"))?;
    let client = connect(&server.addr).await?;
    client
        .rename(context::current(), *id, summary)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to rename the task {id}"))?;
    Ok(())
}

async fn replace(server: &ServerArgs, id: &Uuid, details: String) -> Result<()> {
    let details: TaskDetails = details.parse()?;
    let client = connect(&server.addr).await?;
    client
        .replace(context::current(), *id, details.into())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to replace details of the task {id}"))?;
    Ok(())
}

async fn update(server: &ServerArgs, id: &Uuid, details: String) -> Result<()> {
    let details: TaskDetailsPatch = details.parse()?;
    tracing::debug!("Update task: {details:#?}");
    let details = details.into();
    tracing::debug!(
        "Details as json: {}",
        &serde_json::to_string_pretty(&details).unwrap()
    );
    let client = connect(&server.addr).await?;
    client
        .update(context::current(), *id, details)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to update details of the task {id}"))?;
    Ok(())
}

async fn complete(server: &ServerArgs, id: &Uuid, reopen: bool) -> Result<()> {
    tracing::warn!("task({id}): {}", if reopen { "reopened" } else { "closed" });
    let client = connect(&server.addr).await?;
    client
        .complete(context::current(), *id, reopen)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to change the status of task {id}"))?;
    Ok(())
}

async fn connect(addr: &SocketAddr) -> Result<TaskServiceClient> {
    tracing::info!("CLI will connect to {}", addr);
    let mut transport = tcp::connect(addr, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let transport = transport
        .await
        .into_diagnostic()
        .wrap_err("failed to connect")?;

    let client = TaskServiceClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

fn init_miette_report() {
    use miette::{GraphicalTheme, MietteHandlerOpts};
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .force_graphical(true)
                .graphical_theme(GraphicalTheme::unicode())
                .terminal_links(true)
                .unicode(true)
                .context_lines(3)
                .tab_width(4)
                .break_words(true)
                .build(),
        )
    }));
}
