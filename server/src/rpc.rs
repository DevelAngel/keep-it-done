use kid_types::Task;
pub use kid_types::TaskService;

use rand::Rng;
use tarpc::context;
use tokio::time::{Duration, sleep};

// This is the type that implements the generated World trait.
// It is the business logic and is used to start the server.
#[derive(Clone)]
pub struct TaskRpcServer;

impl TaskService for TaskRpcServer {
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
