cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use chrono::{TimeDelta, Utc};
        use kid_types::{TaskDetails, TaskInfos};
    }
}

use kid_types::TaskFilter;
use kid_types::TaskPriority;
use kid_types::Uuid;
use kid_types::task;

use leptos::prelude::*;

/*
* Sleep blocking:
* ```rust
* std::thread::sleep(std::time::Duration::from_secs(10));
* ```

* Sleep non-blocking:
* ```rust
* tokio::time::sleep(std::time::Duration::from_secs(10)).await;
* ```
*/

#[server(endpoint = "fetch_tasks")]
pub async fn fetch_task_list(filter: TaskFilter) -> Result<Vec<(Uuid, task::Infos)>, ServerFnError> {
    tracing::info!("fetch task list ({filter:?})");
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let list = cache
        .iter()
        .filter(|(_, task)| match filter {
            TaskFilter::Todo => !task.info().is_done(),
            TaskFilter::Done => task.info().is_done(),
            TaskFilter::HasTimeEstimate => {
                !task.info().is_done() && task.time_estimate().is_some()
            }
            TaskFilter::RecentlyChanged => {
                Utc::now().signed_duration_since(task.info().since().with_timezone(&Utc))
                    <= TimeDelta::hours(24)
            }
        })
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    Ok(list)
}

#[server(endpoint = "fetch_task_details")]
pub async fn fetch_task_details(id: Uuid) -> Result<(Uuid, task::Details), ServerFnError> {
    tracing::info!("fetch details for task id {id}");
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let task = cache
        .get(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    Ok((id, task.details().to_owned()))
}

#[server(endpoint = "add_task")]
pub async fn add_task(summary: String) -> Result<(), ServerFnError> {
    use kid_types::Task;
    tracing::info!("add task with summary {summary}");
    let task = Task::new(summary);
    tracing::debug!("task created: {task:?}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let replaced = cache.add(task);
    tracing::debug!("task replaced: {replaced}");
    Ok(())
}

#[server(endpoint = "delete_task")]
pub async fn delete_task(id: Uuid) -> Result<(), ServerFnError> {
    tracing::info!("delete task with id {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let deleted = cache.remove(&id);
    tracing::debug!("task deleted: {deleted}");
    assert!(deleted, "task was not deleted");
    Ok(())
}

#[server(endpoint = "update_task_priority")]
pub async fn update_task_priority(id: Uuid, priority: Option<TaskPriority>) -> Result<(), ServerFnError> {
    tracing::info!("update priority for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match priority {
        Some(p) => task.set_priority(p),
        None => task.clear_priority(),
    }
    Ok(())
}

#[server(endpoint = "update_task_context")]
pub async fn update_task_context(id: Uuid, context: String) -> Result<(), ServerFnError> {
    tracing::info!("update context for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    if context.is_empty() {
        task.clear_context();
    } else {
        task.set_context(context);
    }
    Ok(())
}

#[server(endpoint = "update_task_notes")]
pub async fn update_task_notes(id: Uuid, notes: String) -> Result<(), ServerFnError> {
    tracing::info!("update notes for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    if notes.is_empty() {
        task.clear_notes();
    } else {
        task.set_notes(notes);
    }
    Ok(())
}

#[server(endpoint = "complete_task")]
pub async fn complete_task(id: Uuid, completed: bool) -> Result<(), ServerFnError> {
    tracing::info!("change status for task with id {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    if completed {
        task.mark_done();
    } else {
        task.mark_todo();
    }
    tracing::debug!("task with id {id}: mark {status}", status = task.status());
    Ok(())
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use kid_types::Uuid;
    use kid_types::server::TaskCache;

    use leptos::prelude::ServerFnError;
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

    pub fn task_not_exist_error(id: &Uuid) -> ServerFnError {
        let msg = format!("task with {id} does not exist");
        tracing::warn!("{msg} | UUIDv{} detected", id.get_version_num());
        ServerFnError::ServerError(msg)
    }
}
