use crate::{Task, TaskWithId};

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<TaskWithId>;
    async fn add(task: Task);
}
