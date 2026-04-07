#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;
pub mod task;

pub use crate::task::{
    Category as TaskCategory,
    Date as TaskDate,
    Summary as TaskSummary,
    Filter as TaskFilter, Priority as TaskPriority, Status as TaskStatus, Task,
    TimeEstimate as TaskTimeEstimate,
};
pub use chrono::{DateTime, FixedOffset, Utc};
pub use uuid::Uuid;

use std::borrow::Cow;
use std::time::SystemTime;

pub trait TaskId<'a> {
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
    fn rename(&'a mut self, summary: TaskSummary);
    fn status(&'a self) -> &'a TaskStatus;
    fn change_status(&'a mut self, status: TaskStatus);
    fn category(&'a self) -> &'a str;
    fn set_category(&'a mut self, category: TaskCategory);

    //provides
    fn is_done(&'a self) -> bool {
        match self.status() {
            TaskStatus::ToDo { since: _ } => false,
            TaskStatus::Done { since: _ } => true,
        }
    }
    fn mark_done(&'a mut self) {
        let since = Utc::now().fixed_offset();
        self.change_status(TaskStatus::Done { since });
    }
    fn mark_todo(&'a mut self) {
        let since = Utc::now().fixed_offset();
        self.change_status(TaskStatus::ToDo { since });
    }
    fn since(&'a self) -> &'a DateTime<FixedOffset> {
        match self.status() {
            TaskStatus::ToDo { since } | TaskStatus::Done { since } => since
        }
    }
}

pub trait TaskDetails<'a> {
    fn priority(&'a self) -> Option<&'a TaskPriority>;
    fn set_priority(&'a mut self, priority: TaskPriority);
    fn clear_priority(&'a mut self);
    fn due_date(&'a self) -> Option<&'a TaskDate>;
    fn set_due_date(&'a mut self, due_date: TaskDate);
    fn clear_due_date(&'a mut self);
    fn start_date(&'a self) -> Option<&'a TaskDate>;
    fn set_start_date(&'a mut self, date: TaskDate);
    fn clear_start_date(&'a mut self);
    fn time_estimate(&'a self) -> Option<&'a TaskTimeEstimate>;
    fn set_time_estimate(&mut self, time: TaskTimeEstimate);
    fn clear_time_estimate(&mut self);
    fn notes(&'a self) -> Option<Cow<'a, str>>;
    fn set_notes<T: ToString>(&'a mut self, text: T);
    fn clear_notes(&'a mut self);
}
