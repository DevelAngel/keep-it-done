use crate::task::Details as TaskDetails;
use crate::task::DetailsPatch as TaskDetailsPatch;
use crate::{Task, TaskCategory, TaskContext, TaskSummary, Uuid};
use indexmap::IndexSet;

use tarpc::service;

#[service]
pub trait TaskService {
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task, actor: String);
    async fn rename(id: Uuid, summary: TaskSummary, actor: String);
    async fn replace(id: Uuid, details: TaskDetails, actor: String);
    async fn update(id: Uuid, details: TaskDetailsPatch, actor: String);
    async fn complete(id: Uuid, reopen: bool, actor: String);
    async fn categories() -> Vec<TaskCategory>;
    async fn contexts() -> Vec<TaskContext>;
    async fn recategorize(id: Uuid, category: TaskCategory, actor: String);
    async fn replace_contexts(id: Uuid, contexts: IndexSet<TaskContext>, actor: String);
    async fn add_contexts(id: Uuid, contexts: IndexSet<TaskContext>, actor: String);
}
