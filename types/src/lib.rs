#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
pub use uuid::Uuid;

pub trait TaskWithIdProperties<'a>: TaskProperties<'a> {
    fn id(&'a self) -> &'a Uuid;
    fn created(&'a self) -> DateTime<Utc>;
}

pub trait IdProperties<'a> {
    fn id(&'a self) -> &'a Uuid;
    fn created(&'a self) -> DateTime<Utc>;
}

pub trait TaskProperties<'a> {
    fn completed(&'a self) -> bool;
    fn summary(&'a self) -> &'a str;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskWithId {
    id: Uuid,
    task: Task,
}

impl<'a> TaskWithIdProperties<'a> for TaskWithId {
    fn id(&'a self) -> &'a Uuid {
        &self.id
    }
    fn created(&'a self) -> DateTime<Utc> {
        assert_eq!(self.id.get_version_num(), 7);
        let timestamp = self.id.get_timestamp().expect("UUID v7 expected");
        let timestamp: SystemTime = timestamp.into();
        timestamp.into()
    }
}

impl<'a> TaskProperties<'a> for TaskWithId {
    fn completed(&'a self) -> bool {
        self.task.completed()
    }
    fn summary(&'a self) -> &'a str {
        &self.task.summary()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    completed: bool,
    summary: String,
}

impl<'a> TaskProperties<'a> for Task {
    fn completed(&'a self) -> bool {
        self.completed
    }
    fn summary(&'a self) -> &'a str {
        &self.summary
    }
}

impl Task {
    pub fn new<T: ToString>(summary: T) -> Self {
        Self {
            completed: false,
            summary: summary.to_string(),
        }
    }

    pub fn complete(&mut self, completed: bool) {
        self.completed = completed;
    }
}
