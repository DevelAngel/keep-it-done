#[cfg(feature = "ssr")]
use kid_types::server::TaskList;
use kid_types::{TaskWithId, Uuid};

use leptos::prelude::*;

#[server]
pub async fn fetch_task_list() -> Result<Vec<TaskWithId>, ServerFnError> {
    tracing::info!("fetch task list");
    let cache = ssr::use_task_cache();
    let cache = cache.read().await;
    Ok(cache.to_vec())
}

#[server]
pub async fn add_task(summary: String) -> Result<(), ServerFnError> {
    use kid_types::Task;
    tracing::info!("add task with summary {summary}");
    let task = Task::new(summary);
    tracing::debug!("task created: {task:?}");
    let cache = ssr::use_task_cache();
    let mut cache = cache.write().await;
    let replaced = cache.add(task);
    tracing::debug!("task replaced: {replaced}");
    Ok(())
}

#[server]
pub async fn delete_task(id: Uuid) -> Result<(), ServerFnError> {
    tracing::info!("delete task with id {id}");
    let cache = ssr::use_task_cache();
    let mut cache = cache.write().await;
    let deleted = cache.remove(id);
    tracing::debug!("task deleted: {deleted}");
    assert!(deleted, "task was not deleted");
    Ok(())
}

#[server]
pub async fn complete_task(id: Uuid, completed: bool) -> Result<(), ServerFnError> {
    todo!()
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use kid_types::server::TaskCache;

    use tokio::sync::RwLock;

    use std::sync::Arc;

    pub type SharedTaskCache = Arc<RwLock<TaskCache>>;

    pub fn use_task_cache() -> SharedTaskCache {
        use leptos::context::use_context;
        let Some(cache) = use_context::<SharedTaskCache>() else {
            unreachable!("task cache missing")
        };
        cache
    }
}
