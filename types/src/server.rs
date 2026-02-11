use crate::Uuid;
use crate::{Task, TaskDateEstimation, TaskDetails, TaskPriority, TaskTimeEstimation};

use ahash::RandomState;
use chrono::DateTime;
use indexmap::{IndexMap, IndexSet};

use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Debug)]
pub struct TaskCache {
    deleted: ChangeSet,
    dirty: ChangeSet,
    tasks: DataMap,
}

pub struct TaskMutGuard<'a> {
    id: Uuid,
    dirty: &'a mut ChangeSet,
    task: &'a mut Task,
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
            deleted: ChangeSet::with_hasher(RandomState::new()),
            dirty: ChangeSet::with_hasher(RandomState::new()),
            tasks: DataMap::with_capacity_and_hasher(capacity, RandomState::new()),
        }
    }

    pub fn remove(&mut self, id: &Uuid) -> bool {
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        self.deleted.insert(*id);
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
