#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;

use chrono::{DateTime, Duration, TimeZone, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

use std::borrow::Cow;
use std::time::SystemTime;

pub trait TaskId<'a> {
    // required
    fn id(&'a self) -> &'a Uuid;
    // provided
    fn created(&'a self) -> DateTime<Utc> {
        let id = self.id();
        assert_eq!(id.get_version_num(), 7);
        let timestamp = id.get_timestamp().expect("UUID v7 expected");
        let timestamp: SystemTime = timestamp.into();
        timestamp.into()
    }
}

pub trait TaskInfos<'a> {
    fn summary(&'a self) -> &'a str;
    fn status(&'a self) -> &'a TaskStatus;
    fn change_status(&'a mut self, status: TaskStatus);

    //provides
    fn is_done(&'a self) -> bool {
        match self.status() {
            TaskStatus::ToDo => false,
            TaskStatus::Done => true,
        }
    }
    fn mark_done(&'a mut self) {
        self.change_status(TaskStatus::Done);
    }
    fn mark_todo(&'a mut self) {
        self.change_status(TaskStatus::ToDo);
    }
}

pub trait TaskDetails<'a> {
    // required
    fn due_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>>;
    fn due_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>>;
    fn start_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>>;
    fn start_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>>;
    fn time_estimate(&'a self) -> Option<TaskTimeEstimationRef<'a>>;
    fn context(&'a self) -> Option<&'a str>;
    fn notes(&'a self) -> Option<&'a str>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskWithId {
    id: Uuid,
    task: Task,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    #[serde(flatten)]
    info: TaskInfoProperties,
    #[serde(flatten)]
    details: TaskDetailProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TaskInfoProperties {
    summary: String,
    status: TaskStatus,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct TaskDetailProperties {
    due_date: Option<TaskDateEstimation>,
    start_date: Option<TaskDateEstimation>,
    time_estimate: Option<TaskTimeEstimation>,
    context: Option<String>,
    notes: Option<String>,
}

#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, PartialEq, Eq)]
#[display(rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    ToDo,
    Done,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskDateEstimation {
    Guess(String),
    Precise(DateTime<Utc>),
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum TaskDateEstimationRef<'a, Tz: TimeZone> {
    Guess(&'a str),
    Precise(Cow<'a, DateTime<Tz>>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskTimeEstimation {
    Guess(String),
    Precise(Duration),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskTimeEstimationRef<'a> {
    Guess(&'a str),
    Precise(Cow<'a, Duration>),
}

impl<'a> TaskId<'a> for TaskWithId {
    fn id(&'a self) -> &'a Uuid {
        &self.id
    }
}

impl<'a> TaskInfos<'a> for TaskWithId {
    fn summary(&'a self) -> &'a str {
        &self.task.summary()
    }
    fn status(&'a self) -> &'a TaskStatus {
        self.task.status()
    }
    fn change_status(&'a mut self, status: TaskStatus) {
        self.task.change_status(status);
    }
}

impl<'a> TaskInfos<'a> for Task {
    fn summary(&'a self) -> &'a str {
        &self.info.summary()
    }
    fn status(&'a self) -> &'a TaskStatus {
        self.info.status()
    }
    fn change_status(&'a mut self, status: TaskStatus) {
        self.info.change_status(status);
    }
}

impl<'a> TaskInfos<'a> for TaskInfoProperties {
    fn summary(&'a self) -> &'a str {
        &self.summary
    }
    fn status(&'a self) -> &'a TaskStatus {
        &self.status
    }
    fn change_status(&'a mut self, status: TaskStatus) {
        self.status = status;
    }
}

impl<'a> TaskDetails<'a> for TaskWithId {
    fn due_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>> {
        self.task.due_date()
    }
    fn due_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>> {
        self.task.due_date_with_timezone(tz)
    }
    fn start_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>> {
        self.task.start_date()
    }
    fn start_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>> {
        self.task.start_date_with_timezone(tz)
    }
    fn time_estimate(&'a self) -> Option<TaskTimeEstimationRef<'a>> {
        self.task.time_estimate()
    }
    fn context(&'a self) -> Option<&'a str> {
        self.task.context()
    }
    fn notes(&'a self) -> Option<&'a str> {
        self.task.notes()
    }
}

impl<'a> TaskDetails<'a> for Task {
    fn due_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>> {
        self.details.due_date()
    }
    fn due_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>> {
        self.details.due_date_with_timezone(tz)
    }
    fn start_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>> {
        self.details.start_date()
    }
    fn start_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>> {
        self.details.start_date_with_timezone(tz)
    }
    fn time_estimate(&'a self) -> Option<TaskTimeEstimationRef<'a>> {
        self.details.time_estimate()
    }
    fn context(&'a self) -> Option<&'a str> {
        self.details.context()
    }
    fn notes(&'a self) -> Option<&'a str> {
        self.details.notes()
    }
}

impl<'a> TaskDetails<'a> for TaskDetailProperties {
    fn due_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>> {
        match &self.due_date {
            None => None,
            Some(due_date) => Some(due_date.as_deref()),
        }
    }
    fn due_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>> {
        match &self.due_date {
            None => None,
            Some(due_date) => Some(due_date.as_deref_with_timezone(tz)),
        }
    }
    fn start_date(&'a self) -> Option<TaskDateEstimationRef<'a, Utc>> {
        match &self.due_date {
            None => None,
            Some(date) => Some(date.as_deref()),
        }
    }
    fn start_date_with_timezone<Tz: TimeZone>(
        &'a self,
        tz: &Tz,
    ) -> Option<TaskDateEstimationRef<'a, Tz>> {
        match &self.start_date {
            None => None,
            Some(date) => Some(date.as_deref_with_timezone(tz)),
        }
    }
    fn time_estimate(&'a self) -> Option<TaskTimeEstimationRef<'a>> {
        match &self.time_estimate {
            None => None,
            Some(TaskTimeEstimation::Guess(s)) => Some(TaskTimeEstimationRef::Guess(s)),
            Some(TaskTimeEstimation::Precise(d)) => {
                Some(TaskTimeEstimationRef::Precise(Cow::Borrowed(d)))
            }
        }
    }
    fn context(&'a self) -> Option<&'a str> {
        self.context.as_deref()
    }
    fn notes(&'a self) -> Option<&'a str> {
        self.notes.as_deref()
    }
}

impl<'a> TaskDateEstimation {
    fn as_deref(&'a self) -> TaskDateEstimationRef<'a, Utc> {
        match self {
            TaskDateEstimation::Guess(s) => TaskDateEstimationRef::Guess(s),
            TaskDateEstimation::Precise(dt) => TaskDateEstimationRef::Precise(Cow::Borrowed(dt)),
        }
    }
    fn as_deref_with_timezone<Tz: TimeZone>(&'a self, tz: &Tz) -> TaskDateEstimationRef<'a, Tz> {
        match self {
            TaskDateEstimation::Guess(s) => TaskDateEstimationRef::Guess(s),
            TaskDateEstimation::Precise(dt) => {
                TaskDateEstimationRef::Precise(Cow::Owned(dt.with_timezone(tz)))
            }
        }
    }
}

impl<'a> TaskTimeEstimation {
    #[allow(dead_code)]
    fn as_deref(&'a self) -> TaskTimeEstimationRef<'a> {
        match self {
            TaskTimeEstimation::Guess(s) => TaskTimeEstimationRef::Guess(s),
            TaskTimeEstimation::Precise(dt) => TaskTimeEstimationRef::Precise(Cow::Borrowed(dt)),
        }
    }
}

impl Task {
    pub fn new<T: ToString>(summary: T) -> Self {
        let info = TaskInfoProperties::new(summary);
        let details = TaskDetailProperties::default();
        Self { info, details }
    }
}

impl TaskInfoProperties {
    fn new<T: ToString>(summary: T) -> Self {
        Self {
            summary: summary.to_string(),
            status: TaskStatus::default(),
        }
    }
}
