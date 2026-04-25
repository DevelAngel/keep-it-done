use crate::SharedTaskCache;

pub use kid_types::rpc::TaskService;
use kid_types::task::Details as TaskDetails;
use kid_types::task::DetailsPatch as TaskDetailsPatch;
use kid_types::{Task, TaskCategory, TaskContext, TaskInfos, TaskSummary, Uuid};
use indexmap::IndexSet;
use std::collections::BTreeSet;

use futures::{future, prelude::*};
use miette::{IntoDiagnostic, Result};
use tarpc::context::Context;
use tarpc::serde_transport::tcp;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Json;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub struct RpcServer;

impl RpcServer {
    /// Start RPC server
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
    ) -> Result<()> {
        let actor = format!(
            "ai:{}",
            std::env::var("KID_CLI_USER").unwrap_or_else(|_| "assistant".to_owned())
        );
        tracing::info!(
            "RPC server will listen to: {} (actor: {actor})",
            listener.local_addr().unwrap()
        );
        let mut listener = tcp::listen_on(listener, Json::default)
            .await
            .into_diagnostic()?;
        listener.config_mut().max_frame_length(usize::MAX);
        let stream = listener
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            .map(|channel| {
                let task_cache = task_cache.clone();
                let actor = actor.clone();
                let server = RpcService { task_cache, actor };
                channel.execute(server.serve()).for_each(Self::spawn)
            })
            .buffer_unordered(10);
        tokio::select! {
            _ = stream.for_each(|_| async {}) => {}
            _ = shutdown.cancelled() => {
                tracing::info!("RPC server shutting down");
            }
        }
        Ok(())
    }

    async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
}

#[derive(Clone)]
struct RpcService {
    task_cache: SharedTaskCache,
    actor: String,
}

impl TaskService for RpcService {
    async fn list(self, _: Context) -> Vec<(Uuid, Task)> {
        let task_cache = self.task_cache.read().await;
        task_cache
            .iter()
            .map(|(id, task)| (*id, task.clone()))
            .collect()
    }

    async fn add(self, _: Context, task: Task) {
        let mut task_cache = self.task_cache.write().await;
        task_cache.add(task, &self.actor);
    }

    async fn rename(self, _: Context, id: Uuid, summary: TaskSummary) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            task.rename(summary);
        }
    }

    async fn replace(self, _: Context, id: Uuid, details: TaskDetails) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            task.set_details(details);
        }
    }

    async fn update(self, _: Context, id: Uuid, details: TaskDetailsPatch) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            tracing::debug!("Patch details: {details:#?}");
            task.patch_details(details);
        }
    }

    async fn complete(self, _: Context, id: Uuid, reopen: bool) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            if reopen {
                task.mark_todo();
            } else {
                task.mark_done();
            }
        }
    }

    async fn recategorize(self, _: Context, id: Uuid, category: TaskCategory) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            task.set_category(category);
        }
    }

    async fn replace_contexts(self, _: Context, id: Uuid, contexts: IndexSet<TaskContext>) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            task.set_contexts(contexts);
        }
    }

    async fn add_contexts(self, _: Context, id: Uuid, contexts: IndexSet<TaskContext>) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id, &self.actor) {
            task.extend_contexts(contexts);
        }
    }

    async fn categories(self, _: Context) -> Vec<TaskCategory> {
        let task_cache = self.task_cache.read().await;
        task_cache
            .iter()
            .map(|(_, task)| task.category().parse::<TaskCategory>().unwrap())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    async fn contexts(self, _: Context) -> Vec<TaskContext> {
        let task_cache = self.task_cache.read().await;
        task_cache
            .iter()
            .flat_map(|(_, task)| task.info().contexts().iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
