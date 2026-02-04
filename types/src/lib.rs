#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
pub use uuid::Uuid;

pub trait TaskProperties<'a> {
    fn id(&'a self) -> &'a Uuid;
    fn completed(&'a self) -> bool;
    fn summary(&'a self) -> &'a str;
    fn created(&'a self) -> DateTime<Utc>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskWithId {
    id: Uuid,
    task: Task,
}

impl<'a> TaskProperties<'a> for TaskWithId {
    fn id(&'a self) -> &'a Uuid {
        &self.id
    }
    fn completed(&'a self) -> bool {
        self.task.completed
    }
    fn summary(&'a self) -> &'a str {
        &self.task.summary
    }
    fn created(&'a self) -> DateTime<Utc> {
        assert_eq!(self.id.get_version_num(), 7);
        let timestamp = self.id.get_timestamp().expect("UUID v7 expected");
        let timestamp: SystemTime = timestamp.into();
        timestamp.into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    completed: bool,
    summary: String,
}

impl Task {
    pub fn new<T: ToString>(summary: T) -> Self {
        Self {
            completed: false,
            summary: summary.to_string(),
        }
    }

    pub fn set_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }
}
