mod internal;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use chrono::Utc;
        use kid_types::{TaskDetails, TaskInfos};
        use std::collections::BTreeMap;
        use indexmap::IndexSet;
    }
}

use chrono::{DateTime, FixedOffset, NaiveDate};
use indexmap::IndexMap;

use serde::{Serialize, Deserialize};

use kid_types::TaskAuthors;
use kid_types::TaskAvailability;
use kid_types::TaskCategory;
use kid_types::TaskContext;
use kid_types::TaskDate;
use kid_types::TaskSummary;
use kid_types::TaskPriority;
use kid_types::TaskTimeEstimate;
use kid_types::Uuid;
use kid_types::task;

use leptos::prelude::*;

use internal::{UpcomingGroups, UpcomingBacklog};

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

#[server(endpoint = "fetch_all_open")]
pub async fn fetch_all_open() -> Result<IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut list: Vec<_> = cache
        .iter()
        .filter(|(_, task)| !task.info().is_done())
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    list.sort_by_key(|(id, _)| *id);
    Ok(internal::group_by_category(list))
}

#[server(endpoint = "fetch_what_i_finished")]
pub async fn fetch_what_i_finished() -> Result<IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    Ok(internal::group_finished(cache.iter()))
}

#[server(endpoint = "fetch_quick_wins")]
pub async fn fetch_quick_wins() -> Result<Vec<(TaskTimeEstimate, Vec<(Uuid, task::Infos)>)>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    Ok(internal::group_quick_wins(cache.iter()))
}

/// Fetch open tasks that carry a date, grouped by temporal urgency.
///
/// Returns `(groups, backlog_tasks)` where backlog_tasks are open tasks
/// without any date — unaffected by context filters (UXDR).
#[server(endpoint = "fetch_upcoming")]
pub async fn fetch_upcoming(
    today: NaiveDate,
) -> Result<(UpcomingGroups, UpcomingBacklog), ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    Ok(internal::group_upcoming(cache.iter(), today))
}

/// Per-task metadata for the Recent Changes view.
#[derive(Clone, Serialize, Deserialize)]
pub struct RecentChange {
    pub id: Uuid,
    pub info: task::Infos,
    pub authors: TaskAuthors,
    pub last_changed: DateTime<FixedOffset>,
    pub ai_last: bool,
    pub ai_involved: bool,
}

/// Fetch recently changed tasks.
///
/// Always includes the last 3 calendar days (today + 2).
/// `extra_days` requests additional older days that actually
/// contain data — empty days are skipped.
#[server(endpoint = "fetch_recently_changed")]
pub async fn fetch_recently_changed(extra_days: u32) -> Result<Vec<RecentChange>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let calendar_cutoff = Utc::now().date_naive()
        .checked_sub_days(chrono::Days::new(2)).unwrap();
    let mut all: Vec<_> = cache
        .iter()
        .filter_map(|(id, task)| {
            let last_change = task.authors().values().flatten().max()
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|| task.info().since().with_timezone(&Utc));
            let day = last_change.date_naive();
            let authors = TaskAuthors::from(task.authors());
            let ai_involved = authors.iter().any(|(a, _)| a.starts_with("ai:"));
            let ai_last = authors.iter()
                .max_by_key(|(_, ts)| ts)
                .is_some_and(|(a, _)| a.starts_with("ai:"));
            Some((last_change, day, RecentChange {
                id: id.to_owned(),
                info: task.info().to_owned(),
                authors,
                last_changed: last_change.fixed_offset(),
                ai_last,
                ai_involved,
            }))
        })
        .collect();
    all.sort_by(|(a, _, _), (b, _, _)| b.cmp(a));
    // Always include the 3 calendar days. Beyond that, collect
    // up to `extra_days` distinct older days that have data.
    let mut older_days: IndexSet<chrono::NaiveDate> = IndexSet::new();
    let result = all.into_iter().filter(|(_, day, _)| {
        if *day >= calendar_cutoff {
            true
        } else if older_days.len() < extra_days as usize {
            older_days.insert(*day);
            true
        } else {
            older_days.contains(day)
        }
    }).map(|(_, _, rc)| rc).collect();
    Ok(result)
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
pub async fn fetch_task_details(id: Uuid) -> Result<(Uuid, task::Details, TaskAuthors), ServerFnError> {
    tracing::info!("fetch details for task id {id}");
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let task = cache
        .get(&id)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    let authors = TaskAuthors::from(task.authors());
    Ok((id, task.details().to_owned(), authors))
}

#[server(endpoint = "add_task")]
pub async fn add_task(summary: TaskSummary) -> Result<Uuid, ServerFnError> {
    use kid_types::Task;
    let actor = self::ssr::use_actor().await?;
    tracing::info!("add task with summary {summary}");
    let task = Task::new(summary);
    tracing::debug!("task created: {task:?}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let id = cache.add(task, actor);
    tracing::debug!("task added with id: {id}");
    Ok(id)
}

#[server(endpoint = "rename_task")]
pub async fn rename_task(id: Uuid, summary: TaskSummary) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("rename task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    task.rename(summary);
    Ok(())
}

#[server(endpoint = "delete_task")]
pub async fn delete_task(id: Uuid) -> Result<(), ServerFnError> {
    // no actor needed — delete does not track authorship
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
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update priority for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match priority {
        Some(p) => task.set_priority(p),
        None => task.clear_priority(),
    }
    Ok(())
}

#[server(endpoint = "update_task_time_estimate")]
pub async fn update_task_time_estimate(id: Uuid, estimate: Option<TaskTimeEstimate>) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update time estimate for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match estimate {
        Some(e) => task.set_time_estimate(e),
        None => task.clear_time_estimate(),
    }
    Ok(())
}

#[server(endpoint = "update_task_availability")]
pub async fn update_task_availability(id: Uuid, availability: TaskAvailability) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update availability for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    task.set_availability(availability);
    Ok(())
}

#[server(endpoint = "update_task_due_date")]
pub async fn update_task_due_date(id: Uuid, date: Option<TaskDate>) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update due date for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match date {
        Some(d) => task.set_due_date(d),
        None => task.clear_due_date(),
    }
    Ok(())
}

#[server(endpoint = "update_task_start_date")]
pub async fn update_task_start_date(id: Uuid, date: Option<TaskDate>) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update start date for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    match date {
        Some(d) => task.set_start_date(d),
        None => task.clear_start_date(),
    }
    Ok(())
}

#[server(endpoint = "update_task_category")]
pub async fn update_task_category(id: Uuid, category: TaskCategory) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update category for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    task.set_category(category);
    Ok(())
}

#[server(endpoint = "fetch_contexts")]
pub async fn fetch_contexts() -> Result<Vec<TaskContext>, ServerFnError> {
    let cache = self::ssr::use_task_cache();
    let cache = cache.read().await;
    let mut set: std::collections::BTreeSet<TaskContext> = std::collections::BTreeSet::new();
    for (_, task) in cache.iter() {
        set.extend(task.info().contexts().iter().cloned());
    }
    Ok(set.into_iter().collect())
}

#[server(endpoint = "replace_task_contexts")]
pub async fn replace_task_contexts(id: Uuid, contexts: Vec<TaskContext>) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("replace contexts for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
        .ok_or_else(|| self::ssr::task_not_exist_error(&id))?;
    task.set_contexts(contexts.into_iter().collect::<IndexSet<_>>());
    Ok(())
}

#[server(endpoint = "update_task_notes")]
pub async fn update_task_notes(id: Uuid, notes: String) -> Result<(), ServerFnError> {
    let actor = self::ssr::use_actor().await?;
    tracing::info!("update notes for task {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, &actor)
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
    let actor = self::ssr::use_actor().await?;
    tracing::info!("change status for task with id {id}");
    let cache = self::ssr::use_task_cache();
    let mut cache = cache.write().await;
    let mut task = cache
        .get_mut(&id, actor)
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

    use std::ops::Deref;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Clone, Debug, Default)]
    pub struct SharedTaskCache(Arc<RwLock<TaskCache>>);

    impl SharedTaskCache {
        pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
            let cache = TaskCache::default().with_dir(dir);
            Self(Arc::new(RwLock::new(cache)))
        }
    }

    impl Deref for SharedTaskCache {
        type Target = Arc<RwLock<TaskCache>>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// Fallback actor for web mutations when no
    /// `Remote-User` header is present (e.g. dev mode).
    #[derive(Clone, Debug)]
    pub struct FallbackUser(Option<String>);

    impl FallbackUser {
        pub fn new(user: Option<String>) -> Self {
            Self(user)
        }
    }

    impl std::ops::Deref for FallbackUser {
        type Target = Option<String>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub fn use_task_cache() -> SharedTaskCache {
        use leptos::context::use_context;
        let Some(cache) = use_context::<SharedTaskCache>() else {
            unreachable!("task cache missing")
        };
        cache
    }

    pub async fn use_actor() -> Result<String, ServerFnError> {
        let headers: http::HeaderMap = leptos_axum::extract().await?;
        if let Some(user) = headers
            .get("Remote-User")
            .and_then(|v| v.to_str().ok())
        {
            return Ok(user.to_owned());
        }

        use leptos::context::use_context;
        let Some(fallback) = use_context::<FallbackUser>() else {
            unreachable!("fallback user context missing")
        };
        (*fallback).clone().ok_or_else(|| {
            ServerFnError::ServerError(
                "no authenticated user".into(),
            )
        })
    }

    pub fn task_not_exist_error(id: &Uuid) -> ServerFnError {
        let msg = format!("task with {id} does not exist");
        tracing::warn!("{msg} | UUIDv{} detected", id.get_version_num());
        ServerFnError::ServerError(msg)
    }
}
