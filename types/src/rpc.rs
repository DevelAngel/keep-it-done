use crate::task::Details as TaskDetails;
use crate::task::DetailsPatch as TaskDetailsPatch;
use crate::{Task, TaskCategory, TaskContext, TaskSummary, Uuid};
use indexmap::IndexSet;

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task);
    async fn rename(id: Uuid, summary: TaskSummary);
    async fn replace(id: Uuid, details: TaskDetails);
    async fn update(id: Uuid, details: TaskDetailsPatch);
    async fn complete(id: Uuid, reopen: bool);
    async fn categories() -> Vec<TaskCategory>;
    async fn contexts() -> Vec<TaskContext>;
    async fn recategorize(id: Uuid, category: TaskCategory);
    async fn replace_contexts(id: Uuid, contexts: IndexSet<TaskContext>);
    async fn add_contexts(id: Uuid, contexts: IndexSet<TaskContext>);
}
