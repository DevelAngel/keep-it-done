#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "ssr")]
pub mod server;
pub mod task;
pub mod view;

pub use crate::task::{
    Authors as TaskAuthors,
    Availability as TaskAvailability,
    Category as TaskCategory,
    Context as TaskContext,
    Date as TaskDate,
    Summary as TaskSummary,
    Filter as TaskFilter, Priority as TaskPriority, Status as TaskStatus, Task,
    TimeEstimate as TaskTimeEstimate,
};
pub use chrono::{DateTime, FixedOffset, Utc};
pub use uuid::Uuid;
pub use view::ViewSlug;

use indexmap::IndexSet;

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
    fn contexts(&'a self) -> &'a IndexSet<TaskContext>;
    fn set_contexts(&'a mut self, contexts: IndexSet<TaskContext>);
    fn extend_contexts(&'a mut self, contexts: IndexSet<TaskContext>);
    fn priority(&'a self) -> Option<&'a TaskPriority>;
    fn set_priority(&'a mut self, priority: TaskPriority);
    fn clear_priority(&'a mut self);

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
    fn due_date(&'a self) -> Option<&'a TaskDate>;
    fn set_due_date(&'a mut self, due_date: TaskDate);
    fn clear_due_date(&'a mut self);
    fn start_date(&'a self) -> Option<&'a TaskDate>;
    fn set_start_date(&'a mut self, date: TaskDate);
    fn clear_start_date(&'a mut self);
    fn time_estimate(&'a self) -> Option<&'a TaskTimeEstimate>;
    fn set_time_estimate(&mut self, time: TaskTimeEstimate);
    fn clear_time_estimate(&mut self);
    fn availability(&'a self) -> &'a TaskAvailability;
    fn set_availability(&'a mut self, availability: TaskAvailability);
    fn notes(&'a self) -> Option<Cow<'a, str>>;
    fn set_notes<T: ToString>(&'a mut self, text: T);
    fn clear_notes(&'a mut self);

    // provided

    /// Earliest date the user should begin working on this task.
    ///
    /// Returns `start_date` when set (manual override). Otherwise
    /// computes `due_date` minus `lead_days` eligible days (based on
    /// `availability`). Falls back to `due_date` when `time_estimate`
    /// is absent, and to `None` when `due_date` is also absent.
    fn attention_date(&'a self) -> Option<chrono::NaiveDate> {
        // Manual start_date always wins.
        if let Some(start) = self.start_date() {
            return Some(start.date.date_naive());
        }

        let due = self.due_date()?.date.date_naive();

        let lead = match self.time_estimate() {
            Some(est) => est.lead_days(),
            None => return Some(due),
        };

        if lead == 0 {
            return Some(due);
        }

        // Count backward `lead` eligible days from due (exclusive).
        let avail = *self.availability();
        let mut remaining = lead;
        let mut date = due;
        while remaining > 0 {
            date = date.pred_opt().expect("date underflow");
            if avail.is_eligible(date) {
                remaining -= 1;
            }
        }
        Some(date)
    }
}
