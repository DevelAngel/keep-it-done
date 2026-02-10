use crate::{Task, Uuid};

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task);
}
