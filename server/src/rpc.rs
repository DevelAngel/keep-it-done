use crate::SharedTaskCache;

pub use kid_types::rpc::TaskService;
use kid_types::{Task, TaskInfos, TaskStatus, Uuid};

use futures::{future, prelude::*};
use miette::{IntoDiagnostic, Result};
use rand::Rng;
use tarpc::context;
use tarpc::serde_transport::tcp;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Json;
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

pub struct RpcServer;

impl RpcServer {
    /// Start RPC server
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
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
                let server = RpcService { task_cache };
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
}

impl TaskService for RpcService {
    async fn list(self, _: context::Context) -> Vec<(Uuid, Task)> {
        let sleep_time = {
            let mut rng = rand::rng();
            let sleep_time = rng.random_range(1..10);
            Duration::from_millis(sleep_time)
        };
        sleep(sleep_time).await;

        let task_cache = self.task_cache.read().await;
        task_cache
            .iter()
            .map(|(id, task)| (*id, task.clone()))
            .collect()
    }

    async fn add(self, _: context::Context, task: Task) {
        let mut task_cache = self.task_cache.write().await;
        task_cache.add(task);
    }

    async fn complete(self, _: context::Context, id: Uuid, status: TaskStatus) {
        let mut task_cache = self.task_cache.write().await;
        if let Some(mut task) = task_cache.get_mut(&id) {
            match status {
                TaskStatus::ToDo => task.mark_todo(),
                TaskStatus::Done => task.mark_done(),
            }
        }
    }
}
