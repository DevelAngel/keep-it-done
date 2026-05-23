use crate::task::Details as TaskDetails;
use crate::task::DetailsPatch as TaskDetailsPatch;
use crate::{Task, TaskCategory, TaskContext, TaskPriority, TaskSummary, Uuid};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::path::PathBuf;

use tarpc::service;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SwitchDirError {
    #[error("flush failed: {0}")]
    Flush(String),
    #[error("load failed: {0}")]
    Load(String),
}

#[service]
pub trait TaskService {
    async fn count() -> usize;
    async fn switch_dir(dir: PathBuf) -> Result<usize, SwitchDirError>;
    async fn list() -> Vec<(Uuid, Task)>;
    async fn add(task: Task, actor: String);
    async fn add_with_id(id: Uuid, task: Task, actor: String);
    async fn rename(id: Uuid, summary: TaskSummary, actor: String);
    async fn replace(id: Uuid, details: TaskDetails, actor: String);
    async fn update(id: Uuid, details: TaskDetailsPatch, actor: String);
    async fn complete(id: Uuid, reopen: bool, actor: String);
    async fn categories() -> Vec<TaskCategory>;
    async fn contexts() -> Vec<TaskContext>;
    async fn recategorize(id: Uuid, category: TaskCategory, actor: String);
    async fn replace_contexts(id: Uuid, contexts: IndexSet<TaskContext>, actor: String);
    async fn add_contexts(id: Uuid, contexts: IndexSet<TaskContext>, actor: String);
    async fn set_priority(id: Uuid, priority: Option<TaskPriority>, actor: String);
    async fn set_time_offset(seconds: i64);
    async fn reset_time_offset();
}
