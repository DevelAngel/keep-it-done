use crate::{Task, TaskStatus, Uuid};

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task);
    async fn complete(id: Uuid, status: TaskStatus);
}
