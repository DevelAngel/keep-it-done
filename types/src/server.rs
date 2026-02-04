use crate::{Task, TaskWithId};

use ahash::RandomState;
use indexmap::IndexMap;
use uuid::Uuid;

pub trait TaskList {
    fn to_vec(&self) -> Vec<TaskWithId>;
    fn add(&mut self, task: Task) -> bool;
    fn remove<T: Into<Uuid>>(&mut self, id: T) -> bool;
}

#[derive(Debug)]
pub struct TaskCache(IndexMap<Uuid, Task, RandomState>);

impl TaskCache {
    fn create_id() -> Uuid {
        let id = Uuid::now_v7();
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        id
    }
}

impl Default for TaskCache {
    fn default() -> Self {
        const CAPACITY: usize = 10;
        let mut collection = Self(IndexMap::with_capacity_and_hasher(
            CAPACITY,
            RandomState::new(),
        ));
        collection.add(Task::new("Task A.1"));
        collection.add(Task::new("Task B.2"));
        collection.add(Task::new("Task C.3"));
        collection
    }
}

impl TaskList for TaskCache {
    fn to_vec(&self) -> Vec<TaskWithId> {
        self.0
            .as_slice()
            .iter()
            .map(|(id, task)| TaskWithId {
                id: id.clone(),
                task: task.clone(),
            })
            .collect()
    }

    fn add(&mut self, task: Task) -> bool {
        let id = TaskCache::create_id();
        self.0.insert(id, task).is_some()
    }

    fn remove<T: Into<Uuid>>(&mut self, id: T) -> bool {
        let id = id.into();
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        self.0.shift_remove(&id).is_some()
    }
}
