use kid_types::{TaskId, TaskInfos};

use std::fmt::{self, Display, Formatter};

pub struct TaskPrint<'a, T: TaskId<'a>, U: TaskInfos<'a>>(pub &'a (T, U));

impl<'a, T: TaskId<'a>, U: TaskInfos<'a>> Display for TaskPrint<'a, T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let id = self.0.0.id();
        let summary = self.0.1.summary();
        write!(f, "Task({id}): {summary}")
    }
}
