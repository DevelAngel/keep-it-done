use crate::task::Details as TaskDetails;
use crate::task::DetailsPatch as TaskDetailsPatch;
use crate::{Task, Uuid};

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task);
    async fn rename(id: Uuid, summary: String);
    async fn replace(id: Uuid, details: TaskDetails);
    async fn update(id: Uuid, details: TaskDetailsPatch);
    async fn complete(id: Uuid, reopen: bool);
}
