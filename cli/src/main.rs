use kid_cli::cli::{Cli, Commands, Parser, ServerArgs, StatusFilter};
use kid_cli::connect;
use kid_cli::task::{TaskDetails, TaskDetailsPatch, TaskPrint};

use kid_types::{Task, TaskCategory, TaskContext, TaskInfos, TaskPriority, TaskSummary, Uuid};
use indexmap::IndexSet;

use miette::{IntoDiagnostic, Result, WrapErr};
use schemars::SchemaGenerator;
use tarpc::context;
use tracing::Instrument;

use std::fs::File;
use std::io::{self, BufWriter};
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
            search,
            status,
            pretty,
        } => {
            list(&server, search.as_deref(), status, pretty).await?;
        }
        Commands::Add {
            server,
            actor,
            summary,
            category,
            contexts,
            details,
        } => {
            add(&server, &actor.build(), summary, category, contexts, details.as_deref()).await?;
        }
        Commands::Rename {
            server,
            actor,
            id,
            summary,
        } => {
            rename(&server, &actor.build(), &id, summary).await?;
        }
        Commands::Replace {
            server,
            actor,
            id,
            details,
        } => {
            replace(&server, &actor.build(), &id, details).await?;
        }
        Commands::Update {
            server,
            actor,
            id,
            details,
        } => {
            update(&server, &actor.build(), &id, details).await?;
        }
        Commands::Recategorize { server, actor, id, category } => {
            recategorize(&server, &actor.build(), &id, category).await?;
        }
        Commands::AddContexts { server, actor, id, contexts } => {
            add_contexts(&server, &actor.build(), &id, contexts).await?;
        }
        Commands::ReplaceContexts { server, actor, id, contexts } => {
            replace_contexts(&server, &actor.build(), &id, contexts).await?;
        }
        Commands::Categories { server } => {
            categories(&server).await?;
        }
        Commands::Contexts { server } => {
            contexts(&server).await?;
        }
        Commands::SetPriority { server, actor, id, priority } => {
            set_priority(&server, &actor.build(), &id, priority.as_deref()).await?;
        }
        Commands::Complete { server, actor, id, reopen } => {
            complete(&server, &actor.build(), &id, reopen).await?;
        }
    }
    Ok(())
}

async fn schema(pretty: bool, outfile: Option<&Path>) -> Result<()> {
    use kid_cli::task::Details as TaskDetails;
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

async fn list(server: &ServerArgs, search: Option<&str>, status: StatusFilter, pretty: bool) -> Result<()> {
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

    // Filter by status
    let task_list: Vec<_> = task_list
        .into_iter()
        .filter(|(_, task)| match status {
            StatusFilter::All => true,
            StatusFilter::Open => !task.is_done(),
            StatusFilter::Done => task.is_done(),
        })
        .collect();

    // Apply fuzzy search
    let task_list: Vec<_> = match search.map(str::trim).filter(|q| !q.is_empty()) {
        Some(query) => {
            let words: Vec<&str> = query.split_whitespace().collect();
            task_list
                .into_iter()
                .filter(|(_, task)| {
                    let haystack = format!(
                        "{} {} {}",
                        task.summary(),
                        task.category(),
                        task.contexts()
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>()
                            .join(" "),
                    );
                    words
                        .iter()
                        .all(|w| sublime_fuzzy::best_match(w, &haystack).is_some())
                })
                .collect()
        }
        None => task_list,
    };

    // JSON output
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

    Ok(())
}

async fn add(server: &ServerArgs, actor: &str, summary: String, category: String, contexts: Vec<String>, details: Option<&str>) -> Result<()> {
    let summary: TaskSummary = summary.parse().map_err(|e| miette::miette!("{e}"))?;
    let category: TaskCategory = category.parse().map_err(|e| miette::miette!("{e}"))?;
    let contexts: IndexSet<TaskContext> = contexts
        .iter()
        .map(|s| s.parse().map_err(|e| miette::miette!("{e}")))
        .collect::<Result<_>>()?;
    let mut task = Task::new(summary).with_category(category).with_contexts(contexts);
    if let Some(details) = details {
        let details: TaskDetails = details.parse()?;
        task = task.with_details(details.into());
    }
    tracing::debug!("task created:\n{task:#?}");

    let client = connect(&server.addr).await?;
    client
        .add(context::current(), task, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err("failed to add task")?;
    Ok(())
}

async fn rename(server: &ServerArgs, actor: &str, id: &Uuid, summary: String) -> Result<()> {
    let summary: TaskSummary = summary.parse().map_err(|e| miette::miette!("{e}"))?;
    let client = connect(&server.addr).await?;
    client
        .rename(context::current(), *id, summary, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to rename the task {id}"))?;
    Ok(())
}

async fn replace(server: &ServerArgs, actor: &str, id: &Uuid, details: String) -> Result<()> {
    let details: TaskDetails = details.parse()?;
    let client = connect(&server.addr).await?;
    client
        .replace(context::current(), *id, details.into(), actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to replace details of the task {id}"))?;
    Ok(())
}

async fn update(server: &ServerArgs, actor: &str, id: &Uuid, details: String) -> Result<()> {
    let details: TaskDetailsPatch = details.parse()?;
    tracing::debug!("Update task: {details:#?}");
    let details = details.into();
    tracing::debug!(
        "Details as json: {}",
        &serde_json::to_string_pretty(&details).unwrap()
    );
    let client = connect(&server.addr).await?;
    client
        .update(context::current(), *id, details, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to update details of the task {id}"))?;
    Ok(())
}

async fn recategorize(server: &ServerArgs, actor: &str, id: &Uuid, category: String) -> Result<()> {
    let category: TaskCategory = category.parse().map_err(|e| miette::miette!("{e}"))?;
    let client = connect(&server.addr).await?;
    client
        .recategorize(context::current(), *id, category, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to recategorize task {id}"))?;
    Ok(())
}

async fn add_contexts(server: &ServerArgs, actor: &str, id: &Uuid, contexts: Vec<String>) -> Result<()> {
    let contexts: IndexSet<TaskContext> = contexts
        .iter()
        .map(|s| s.parse().map_err(|e| miette::miette!("{e}")))
        .collect::<Result<_>>()?;
    let client = connect(&server.addr).await?;
    client
        .add_contexts(context::current(), *id, contexts, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to add contexts to task {id}"))?;
    Ok(())
}

async fn replace_contexts(server: &ServerArgs, actor: &str, id: &Uuid, contexts: Vec<String>) -> Result<()> {
    let contexts: IndexSet<TaskContext> = contexts
        .iter()
        .map(|s| s.parse().map_err(|e| miette::miette!("{e}")))
        .collect::<Result<_>>()?;
    let client = connect(&server.addr).await?;
    client
        .replace_contexts(context::current(), *id, contexts, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to change contexts of task {id}"))?;
    Ok(())
}

async fn contexts(server: &ServerArgs) -> Result<()> {
    let client = connect(&server.addr).await?;
    let ctxs = client
        .contexts(context::current())
        .await
        .into_diagnostic()
        .wrap_err("failed to fetch contexts")?;
    for ctx in ctxs {
        println!("{ctx}");
    }
    Ok(())
}

async fn categories(server: &ServerArgs) -> Result<()> {
    let client = connect(&server.addr).await?;
    let cats = client
        .categories(context::current())
        .await
        .into_diagnostic()
        .wrap_err("failed to fetch categories")?;
    for cat in cats {
        println!("{cat}");
    }
    Ok(())
}

async fn set_priority(server: &ServerArgs, actor: &str, id: &Uuid, priority: Option<&str>) -> Result<()> {
    let priority: Option<TaskPriority> = priority
        .map(|s| s.parse().map_err(|e| miette::miette!("{e}")))
        .transpose()?;
    let client = connect(&server.addr).await?;
    client
        .set_priority(context::current(), *id, priority, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to set priority of task {id}"))?;
    Ok(())
}

async fn complete(server: &ServerArgs, actor: &str, id: &Uuid, reopen: bool) -> Result<()> {
    tracing::warn!("task({id}): {}", if reopen { "reopened" } else { "closed" });
    let client = connect(&server.addr).await?;
    client
        .complete(context::current(), *id, reopen, actor.to_owned())
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to change the status of task {id}"))?;
    Ok(())
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
