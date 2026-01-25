use kid_types::Task;
pub use kid_types::rpc::TaskService;

use anyhow::Result;
use futures::{future, prelude::*};
use rand::Rng;
use tarpc::context;
use tarpc::serde_transport::tcp;
use tarpc::server::{self, Channel};
use tarpc::tokio_serde::formats::Json;
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};

pub struct RpcServer;

impl RpcServer {
    /// Start RPC server
    pub async fn serve(listener: TcpListener) -> Result<()> {
        tracing::info!(
            "RPC Server will listen to: {}",
            listener.local_addr().unwrap()
        );
        let mut listener = tcp::listen_on(listener, Json::default).await?;
        listener.config_mut().max_frame_length(usize::MAX);
        listener
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            .map(|channel| {
                let server = RpcService;
                channel.execute(server.serve()).for_each(Self::spawn)
            })
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;
        Ok(())
    }

    async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
}

#[derive(Clone)]
struct RpcService;

impl TaskService for RpcService {
    async fn list(self, _: context::Context) -> Vec<Task> {
        let sleep_time = {
            let mut rng = rand::rng();
            let sleep_time = rng.random_range(1..10);
            Duration::from_millis(sleep_time)
        };
        sleep(sleep_time).await;

        const MY_TASK_THIRD: &str = "my third task";
        let task_list = vec![
            Task::new("my frist task"),
            Task::new("my second task".to_string()),
            Task::new(MY_TASK_THIRD),
        ];
        task_list
    }
}
