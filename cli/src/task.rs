use kid_types::TaskProperties;

use std::fmt::{self, Display, Formatter};

pub struct TaskPrint<'a, T: TaskProperties<'a>>(pub &'a T);

impl<'a, T: TaskProperties<'a>> Display for TaskPrint<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let id = self.0.id();
        let summary = self.0.summary();
        write!(f, "Task({id}): {summary}")
    }
}
