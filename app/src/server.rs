cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use chrono::{TimeDelta, Utc};
        use kid_types::{TaskDetails, TaskInfos};
        use std::collections::BTreeMap;
    }
}

use indexmap::IndexMap;

use kid_types::TaskCategory;
use kid_types::TaskDate;
use kid_types::TaskSummary;
use kid_types::TaskPriority;
use kid_types::TaskTimeEstimate;
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

#[server(endpoint = "fetch_my_day")]
pub async fn fetch_my_day() -> Result<IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut list: Vec<_> = cache
        .iter()
        .filter(|(_, task)| !task.info().is_done())
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    list.sort_by_key(|(id, _)| *id);
    Ok(group_by_category(list))
}

#[server(endpoint = "fetch_what_i_finished")]
pub async fn fetch_what_i_finished() -> Result<IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut list: Vec<_> = cache
        .iter()
        .filter(|(_, task)| task.info().is_done())
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    list.sort_by(|(_, a), (_, b)| b.since().cmp(a.since()));
    Ok(group_by_category(list))
}

#[server(endpoint = "fetch_quick_wins")]
pub async fn fetch_quick_wins() -> Result<Vec<(Uuid, task::Infos)>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut list: Vec<_> = cache
        .iter()
        .filter(|(_, task)| !task.info().is_done() && task.time_estimate().is_some())
        .map(|(id, task)| (id.to_owned(), task.info().to_owned(), task.time_estimate().cloned()))
        .collect();
    list.sort_by_key(|(_, _, te)| *te);
    Ok(list.into_iter().map(|(id, info, _)| (id, info)).collect())
}

#[server(endpoint = "fetch_recently_changed")]
pub async fn fetch_recently_changed() -> Result<Vec<(Uuid, task::Infos)>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut list: Vec<_> = cache
        .iter()
        .filter(|(_, task)| {
            Utc::now().signed_duration_since(task.info().since().with_timezone(&Utc))
                <= TimeDelta::hours(24)
        })
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    list.sort_by(|(_, a), (_, b)| b.since().cmp(a.since()));
    Ok(list)
}

#[cfg(feature = "ssr")]
fn group_by_category(list: Vec<(Uuid, task::Infos)>) -> IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>> {
    let mut btree: BTreeMap<TaskCategory, Vec<(Uuid, task::Infos)>> = BTreeMap::new();
    for item in list {
        btree.entry(item.1.category().parse().unwrap()).or_default().push(item);
    }
    btree.into_iter().collect()
}

#[server(endpoint = "fetch_categories")]
pub async fn fetch_categories() -> Result<Vec<TaskCategory>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut cats: BTreeMap<TaskCategory, ()> = BTreeMap::new();
    for (_, task) in cache.iter() {
        cats.entry(task.info().category().parse().unwrap()).or_default();
    }
    Ok(cats.into_keys().collect())
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
pub async fn add_task(summary: TaskSummary) -> Result<(), ServerFnError> {
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

#[server(endpoint = "rename_task")]
pub async fn rename_task(id: Uuid, summary: TaskSummary) -> Result<(), ServerFnError> {
    tracing::info!("rename task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    task.rename(summary);
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

#[server(endpoint = "update_task_time_estimate")]
pub async fn update_task_time_estimate(id: Uuid, estimate: Option<TaskTimeEstimate>) -> Result<(), ServerFnError> {
    tracing::info!("update time estimate for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match estimate {
        Some(e) => task.set_time_estimate(e),
        None => task.clear_time_estimate(),
    }
    Ok(())
}

#[server(endpoint = "update_task_due_date")]
pub async fn update_task_due_date(id: Uuid, date: Option<TaskDate>) -> Result<(), ServerFnError> {
    tracing::info!("update due date for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match date {
        Some(d) => task.set_due_date(d),
        None => task.clear_due_date(),
    }
    Ok(())
}

#[server(endpoint = "update_task_start_date")]
pub async fn update_task_start_date(id: Uuid, date: Option<TaskDate>) -> Result<(), ServerFnError> {
    tracing::info!("update start date for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match date {
        Some(d) => task.set_start_date(d),
        None => task.clear_start_date(),
    }
    Ok(())
}

#[server(endpoint = "update_task_category")]
pub async fn update_task_category(id: Uuid, category: TaskCategory) -> Result<(), ServerFnError> {
    tracing::info!("update category for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    task.set_category(category);
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
