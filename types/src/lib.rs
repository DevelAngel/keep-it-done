#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;
pub mod task;

pub use crate::task::{
    DateEstimation as TaskDateEstimation, DateEstimationRef as TaskDateEstimationRef,
    Priority as TaskPriority, Status as TaskStatus, Task, TimeEstimation as TaskTimeEstimation,
    TimeEstimationRef as TaskTimeEstimationRef,
    Filter as TaskFilter,
};
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

use chrono::TimeZone;
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
    fn summary(&'a self) -> Cow<'a, str>;
    fn rename<T: ToString>(&'a mut self, summary: T);
    fn status(&'a self) -> &'a TaskStatus;
    fn change_status(&'a mut self, status: TaskStatus);

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
}

pub trait TaskDetails<'a> {
    fn priority(&'a self) -> Option<&'a TaskPriority>;
    fn set_priority(&'a mut self, priority: TaskPriority);
    fn clear_priority(&'a mut self);
    fn due_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<TaskDateEstimationRef<'a, Tz>>;
    fn set_due_date(&'a mut self, due_date: TaskDateEstimation);
    fn clear_due_date(&'a mut self);
    fn start_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<TaskDateEstimationRef<'a, Tz>>;
    fn set_start_date(&'a mut self, start_date: TaskDateEstimation);
    fn clear_start_date(&'a mut self);
    fn time_estimate(&'a self) -> Option<TaskTimeEstimationRef<'a>>;
    fn set_time_estimate(&'a mut self, time: TaskTimeEstimation);
    fn clear_time_estimate(&'a mut self);
    fn context(&'a self) -> Option<Cow<'a, str>>;
    fn set_context<T: ToString>(&'a mut self, text: T);
    fn clear_context(&'a mut self);
    fn notes(&'a self) -> Option<Cow<'a, str>>;
    fn set_notes<T: ToString>(&'a mut self, text: T);
    fn clear_notes(&'a mut self);
}
