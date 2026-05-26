use crate::cache::SharedEventBus;
use crate::{SharedTaskCache, SharedTimeOffset};
use kid_app::events::{FlushOutcome, ServerEvent};

pub use kid_types::rpc::{SwitchDirError, TaskService};
use kid_types::task::Details as TaskDetails;
use kid_types::task::DetailsPatch as TaskDetailsPatch;
use kid_types::{Task, TaskCategory, TaskContext, TaskInfos, TaskPriority, TaskSummary, Uuid};

use futures::{future, prelude::*};
use indexmap::IndexSet;
use miette::{IntoDiagnostic, Result};
use tarpc::context::Context;
use tarpc::serde_transport::tcp;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Json;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::Level;

use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

pub struct RpcServer;

impl RpcServer {
    /// Start RPC server
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
        time_offset: SharedTimeOffset,
        event_bus: SharedEventBus,
    ) -> Result<()> {
        tracing::info!(
            "RPC server will listen to: {}",
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
                let time_offset = time_offset.clone();
                let event_bus = event_bus.clone();
                let server = RpcService { task_cache, time_offset, event_bus };
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
    time_offset: SharedTimeOffset,
    event_bus: SharedEventBus,
}

impl RpcService {
    /// Return a closure that broadcasts a `TaskChanged` event.
    fn notify_callback(&self) -> impl FnOnce(Uuid, String) + Send + 'static {
        let bus = self.event_bus.clone();
        move |id, actor| {
            let _ = bus.send(ServerEvent::TaskChanged { id, actor });
        }
    }
}

impl TaskService for RpcService {
    async fn count(self, _: Context) -> usize {
        let task_cache = self.task_cache.read().await;
        task_cache.len()
    }

    async fn switch_dir(self, _: Context, dir: PathBuf) -> Result<usize, SwitchDirError> {
        let mut cache = self.task_cache.write().await;
        cache.flush().await.map_err(|e| SwitchDirError::Flush(e.to_string()))?;
        if dir.is_absolute() {
            tracing::info!("new tasks directory: {}", dir.display());
        } else if tracing::enabled!(Level::INFO) {
            tracing::info!("new tasks directory (relative): {}", dir.display());
            let cwd = env::current_dir().expect("CWD available");
            let dir = cwd.join(&dir);
            tracing::info!("new tasks directory (absolute): {}", dir.display());
        }
        cache.reset(dir);
        let (loaded, _) = cache.load().await.map_err(|e| SwitchDirError::Load(e.to_string()))?;
        Ok(loaded)
    }

    async fn list(self, _: Context) -> Vec<(Uuid, Task)> {
        let task_cache = self.task_cache.read().await;
        task_cache
            .iter()
            .map(|(id, task)| (*id, task.clone()))
            .collect()
    }

    async fn add(self, _: Context, task: Task, actor: String) {
        let mut task_cache = self.task_cache.write().await;
        let id = task_cache.add(task, &actor);
        let _ = self.event_bus.send(ServerEvent::TaskChanged { id, actor });
    }

    async fn add_with_id(self, _: Context, id: Uuid, task: Task, actor: String) {
        let mut task_cache = self.task_cache.write().await;
        task_cache.add_with_id(id, task, &actor);
        let _ = self.event_bus.send(ServerEvent::TaskChanged { id, actor });
    }

    async fn rename(self, _: Context, id: Uuid, summary: TaskSummary, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            task.rename(summary);
        }
    }

    async fn replace(self, _: Context, id: Uuid, details: TaskDetails, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            task.set_details(details);
        }
    }

    async fn update(self, _: Context, id: Uuid, details: TaskDetailsPatch, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            tracing::debug!("Patch details: {details:#?}");
            task.patch_details(details);
        }
    }

    async fn complete(self, _: Context, id: Uuid, reopen: bool, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            if reopen {
                task.mark_todo();
            } else {
                task.mark_done();
            }
        }
    }

    async fn recategorize(self, _: Context, id: Uuid, category: TaskCategory, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            task.set_category(category);
        }
    }

    async fn replace_contexts(self, _: Context, id: Uuid, contexts: IndexSet<TaskContext>, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            task.set_contexts(contexts);
        }
    }

    async fn add_contexts(self, _: Context, id: Uuid, contexts: IndexSet<TaskContext>, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            task.extend_contexts(contexts);
        }
    }

    async fn set_priority(self, _: Context, id: Uuid, priority: Option<TaskPriority>, actor: String) {
        let notify = self.notify_callback();
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut_notifying(&id, &actor, notify) {
            match priority {
                Some(p) => task.set_priority(p),
                None => task.clear_priority(),
            }
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

    async fn set_time_offset(self, _: Context, seconds: i64) {
        tracing::info!("setting time offset to {seconds}s");
        self.time_offset.set(seconds);
    }

    async fn reset_time_offset(self, _: Context) {
        tracing::info!("resetting time offset");
        self.time_offset.reset();
    }

    async fn flush(self, _: Context) -> usize {
        tracing::info!("force-flushing task cache");
        let mut cache = self.task_cache.write().await;
        match cache.flush().await {
            Ok(num) => {
                if num > 0 {
                    let _ = self.event_bus.send(ServerEvent::Flush(
                        FlushOutcome::Success { count: num },
                    ));
                }
                num
            }
            Err(e) => {
                tracing::error!("flush failed: {e}");
                let _ = self.event_bus.send(ServerEvent::Flush(
                    FlushOutcome::Error { message: e.to_string() },
                ));
                0
            }
        }
    }
}
