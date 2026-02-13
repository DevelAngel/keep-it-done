use crate::Uuid;
use crate::{Task, TaskDateEstimation, TaskDetails, TaskPriority, TaskTimeEstimation};

use ahash::{HashMap, RandomState};
use chrono::DateTime;
use indexmap::{IndexMap, IndexSet};
use miette::Diagnostic;
use thiserror::Error;
use tokio::fs;

use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug)]
pub struct TaskCache {
    dir: PathBuf,
    tasks: DataMap,
    dirty: ChangeSet,
}

#[derive(Debug)]
pub struct TaskMutGuard<'a> {
    id: Uuid,
    task: &'a mut Task,
    dirty: &'a mut ChangeSet,
}

pub type FlushResult<T> = Result<T, FlushError>;

#[derive(Error, Diagnostic, Debug)]
pub enum FlushError {
    #[cfg(feature = "ssr-test")]
    #[error("task {0}: failed to flush")]
    TestError(Uuid),
    #[error("task {0}: failed to convert to JSON")]
    JsonError(Uuid, #[source] serde_json::Error),
    #[error("task {0}: failed to remove task file")]
    IoRemoveError(Uuid, #[source] io::Error),
    #[error("task {0}: failed to write temporary task file")]
    IoWriteError(Uuid, #[source] io::Error),
    #[error("task {0}: failed to save task file (by renaming the temporary task file)")]
    IoRenameError(Uuid, #[source] io::Error),
    #[error("flushing of {0}/{1} tasks failed")]
    ErrorList(usize, usize, #[related] Vec<FlushError>),
}

type DataMap = IndexMap<Uuid, Task, RandomState>;
type ChangeSet = IndexSet<Uuid, RandomState>;

impl TaskCache {
    fn create_id() -> Uuid {
        let id = Uuid::now_v7();
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        id
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            dir: PathBuf::new(),
            tasks: DataMap::with_capacity_and_hasher(capacity, RandomState::new()),
            dirty: ChangeSet::with_hasher(RandomState::new()),
        }
    }

    #[allow(dead_code)]
    fn with_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.dir = dir.into();
        self
    }

    pub fn remove(&mut self, id: &Uuid) -> bool {
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        self.dirty.insert(*id);
        self.tasks.shift_remove(id).is_some()
    }

    pub fn add(&mut self, task: Task) -> bool {
        let id = TaskCache::create_id();
        self.dirty.insert(id);
        self.tasks.insert(id, task).is_some()
    }

    /*
     * Note: get() is accessable via Deref trait
     */

    pub fn get_mut(&mut self, id: &Uuid) -> Option<TaskMutGuard<'_>> {
        let dirty = &mut self.dirty;
        self.tasks.get_mut(id).map(|task| TaskMutGuard {
            id: *id,
            dirty,
            task,
        })
    }

    pub async fn flush(&mut self) -> FlushResult<usize> {
        let num = self.dirty.len();
        let mut errors = HashMap::default();

        for id in &self.dirty {
            if let Some(task) = self.tasks.get(id) {
                // Task found -> write file
                if let Err(e) = self.write_task_file(id, task).await {
                    errors.insert(*id, e);
                }
            } else {
                // Task deleted -> delete file
                if let Err(e) = self.delete_task_file(id).await {
                    errors.insert(*id, e);
                }
            }
        }

        if errors.is_empty() {
            self.dirty.clear();
            Ok(num)
        } else {
            // remove ids for successfully written files
            for id in self.dirty.clone().iter() {
                if !errors.contains_key(id) {
                    self.dirty.swap_remove(id);
                }
            }
            Err(FlushError::ErrorList(
                self.dirty.len(),
                num,
                errors.into_values().collect(),
            ))
        }
    }

    async fn write_task_file(&self, id: &Uuid, task: &Task) -> FlushResult<()> {
        let path = Self::filename(&self.dir, id);
        let temp_path = path.with_extension("json.tmp");

        let task = serde_json::to_string_pretty(task).map_err(|e| FlushError::JsonError(*id, e))?;
        fs::write(&temp_path, task)
            .await
            .map_err(|e| FlushError::IoWriteError(*id, e))?;
        fs::rename(&temp_path, path)
            .await
            .map_err(|e| FlushError::IoRenameError(*id, e))?;

        cfg_if::cfg_if! {
            if #[cfg(feature = "ssr-test")] {
                Self::should_return_error(id)
            } else {
                Ok(())
            }
        }
    }

    #[cfg(feature = "ssr-test")]
    fn should_return_error(id: &Uuid) -> FlushResult<()> {
        use rand::Rng;
        let mut rng = rand::rng();
        if rng.random::<bool>() {
            Ok(())
        } else {
            Err(FlushError::TestError(*id))
        }
    }

    async fn delete_task_file(&self, id: &Uuid) -> FlushResult<()> {
        fs::remove_file(Self::filename(&self.dir, id))
            .await
            .map_err(|e| FlushError::IoRemoveError(*id, e))?;
        Ok(())
    }

    fn filename<P: AsRef<Path>>(dir: P, id: &Uuid) -> PathBuf {
        let id = id.as_simple();
        dir.as_ref().join(format!("task-{id}.json"))
    }
}

impl Default for TaskCache {
    fn default() -> Self {
        let mut collection = Self::with_capacity(10);

        // some test data ...
        let mut num = 1;
        for _ in 0..3 {
            for alpha in 'A'..='Z' {
                use chrono::{TimeDelta, Utc};
                let mut task = Task::new(format!("Task {alpha}{alpha}.{num:02}"));
                let priority = match alpha {
                    'A' | 'C' | 'F' => Some(TaskPriority::A),
                    'B' | 'E' | 'H' => Some(TaskPriority::B),
                    'D' => Some(TaskPriority::C),
                    _ => None,
                };
                let due_date = match alpha {
                    'C' | 'F' => Some(TaskDateEstimation::Guess("Mitte 2026".to_owned())),
                    'A' | 'E' => Some(TaskDateEstimation::Precise(
                        DateTime::parse_from_rfc3339("2026-12-19T16:39:57-08:00")
                            .expect("due date rfc3339"),
                    )),
                    'G' => Some(TaskDateEstimation::Precise(
                        DateTime::parse_from_str("2026-05-31 12:00 +01", "%F %R %#z")
                            .expect("due date y-m-d"),
                    )),
                    'B' => Some(TaskDateEstimation::Precise(
                        (Utc::now() + TimeDelta::days(3) - TimeDelta::minutes(30)).into(),
                    )),
                    'D' => Some(TaskDateEstimation::Precise(
                        (Utc::now() - TimeDelta::days(2) - TimeDelta::hours(5)).into(),
                    )),
                    _ => None,
                };
                let start_date = match alpha {
                    'B' | 'C' | 'E' => Some(TaskDateEstimation::Guess("Mitte 2026".to_owned())),
                    'A' | 'F' => Some(TaskDateEstimation::Precise(
                        DateTime::parse_from_rfc3339("2026-11-19T16:39:57-08:00")
                            .expect("start date rfc3339"),
                    )),
                    'G' => Some(TaskDateEstimation::Precise(
                        DateTime::parse_from_str("2026-05-02 12:00 +02", "%F %R %#z")
                            .expect("start date y-m-d"),
                    )),
                    _ => None,
                };
                let time_estimate = match alpha {
                    'A' | 'C' | 'H' => Some(TaskTimeEstimation::Guess("ein WE".to_owned())),
                    'E' | 'F' => Some(TaskTimeEstimation::Precise(Duration::from_hours(3))),
                    'G' => Some(TaskTimeEstimation::Precise(Duration::from_mins(30))),
                    _ => None,
                };
                let context = match alpha {
                    'A' | 'D' | 'E' => Some("Kitchen"),
                    'B' | 'H' => Some("Couch"),
                    _ => None,
                };
                let notes = match alpha {
                    'A' | 'B' | 'E' => Some("Meine Notizen"),
                    'D' | 'H' => Some("Bli Bla Blupp"),
                    _ => None,
                };
                if let Some(p) = priority {
                    task.set_priority(p);
                }
                if let Some(d) = due_date {
                    task.set_due_date(d);
                }
                if let Some(d) = start_date {
                    task.set_start_date(d);
                }
                if let Some(t) = time_estimate {
                    task.set_time_estimate(t);
                }
                if let Some(t) = context {
                    task.set_context(t);
                }
                if let Some(t) = notes {
                    task.set_notes(t);
                }
                collection.add(task);
                num += 1;
            }
        }
        collection
    }
}

impl Deref for TaskCache {
    type Target = DataMap;
    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

impl<'a> Deref for TaskMutGuard<'a> {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        self.task
    }
}

impl<'a> DerefMut for TaskMutGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.task
    }
}

impl<'a> Drop for TaskMutGuard<'a> {
    fn drop(&mut self) {
        self.dirty.insert(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::TempDir;
    use miette::{IntoDiagnostic, MietteHandlerOpts, Result};
    use std::path::PathBuf;
    use tracing_test::traced_test;
    use uuid::uuid;

    #[test]
    fn filenames() {
        // empty dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::new(),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("task-019c500fe59875a19bd9286e3d82cd04.json")
        );
        // absolute dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::from("/tasks"),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("/tasks/task-019c500fe59875a19bd9286e3d82cd04.json")
        );
        // relative dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::from("tasks"),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("tasks/task-019c500fe59875a19bd9286e3d82cd04.json")
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn flush() -> Result<()> {
        init_miette_report();
        let dir = TempDir::with_prefix("kid-").into_diagnostic()?;
        tracing::debug!("dir: {}", dir.path().display());
        assert_files(&dir, 0)?;
        let mut cache = TaskCache::default().with_dir(dir.path());
        assert_files(&dir, 0)?;
        cache.flush().await?;
        assert_files(&dir, cache.len())?;
        dir.close().into_diagnostic()?;
        Ok(())
    }

    fn assert_files(dir: &Path, num: usize) -> Result<()> {
        let entries = std::fs::read_dir(dir).into_diagnostic()?;
        assert_eq!(entries.count(), num);
        Ok(())
    }

    fn init_miette_report() {
        let _ = miette::set_hook(Box::new(|_| {
            Box::new(
                MietteHandlerOpts::new()
                    .color(false)
                    .without_syntax_highlighting()
                    .terminal_links(true)
                    .unicode(true)
                    .build(),
            )
        }));
    }
}
