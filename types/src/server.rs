use crate::Task;

use indexmap::IndexSet;
use uuid::Uuid;

use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

impl Task {
    pub fn new<T: ToString>(summary: T) -> Self {
        Self::default().summary(summary)
    }

    fn with_id(id: Uuid) -> Self {
        let summary = "".to_owned();
        Self { id, summary }
    }

    pub fn summary<T: ToString>(mut self, summary: T) -> Self {
        self.summary = summary.to_string();
        self
    }
}

impl Default for Task {
    fn default() -> Self {
        let id = Uuid::now_v7();
        let summary = "".to_owned();
        Self { id, summary }
    }
}

#[derive(Debug, Clone, Eq)]
struct CachedTask(Task);

impl Deref for CachedTask {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CachedTask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq for CachedTask {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

impl Hash for CachedTask {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub trait TaskList {
    fn to_vec(&self) -> Vec<Task>;
    fn add(&mut self, task: Task) -> bool;
    fn remove<T: Into<Uuid>>(&mut self, id: T) -> bool;
}

#[derive(Debug)]
pub struct TaskCache(IndexSet<CachedTask>);

impl Default for TaskCache {
    fn default() -> Self {
        let mut set = IndexSet::with_capacity(10);
        let list = vec![
            Task::new("Task A.1"),
            Task::new("Task B.2"),
            Task::new("Task C.3"),
        ];
        list.into_iter().for_each(|task| {
            set.insert(CachedTask(task));
        });
        Self(set)
    }
}

impl TaskList for TaskCache {
    fn to_vec(&self) -> Vec<Task> {
        self.0.iter().map(|t| t.deref()).cloned().collect()
    }

    fn add(&mut self, task: Task) -> bool {
        self.0.insert(CachedTask(task))
    }

    fn remove<T: Into<Uuid>>(&mut self, id: T) -> bool {
        let id = id.into();
        self.0.shift_remove(&CachedTask(Task::with_id(id)))
    }
}
