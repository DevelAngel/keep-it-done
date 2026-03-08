pub use kid_types::task::{Details, DetailsPatch};
use kid_types::{TaskId, TaskInfos};

use miette::{Diagnostic, Result, SourceOffset};
use serde::Serialize;
use thiserror::Error;

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct TaskPrint<'a, T: TaskId<'a>, U: TaskInfos<'a>> {
    id: &'a T,
    task: &'a U,
}

impl<'a, T: TaskId<'a>, U: TaskInfos<'a>> TaskPrint<'a, T, U> {
    pub fn new(id: &'a T, task: &'a U) -> Self {
        Self { id, task }
    }
}

impl<'a, T: TaskId<'a>, U: TaskInfos<'a>> Display for TaskPrint<'a, T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let id = self.id.id();
        let summary = self.task.summary();
        write!(f, "Task({id}): {summary}")
    }
}

#[derive(Debug)]
pub struct TaskDetails(Details);

#[derive(Debug)]
pub struct TaskDetailsPatch(DetailsPatch);

impl FromStr for TaskDetails {
    type Err = ParsingError;
    fn from_str(val: &str) -> ParsingResult<Self> {
        let val = serde_json::from_str(val).map_err(|e| ParsingError::from_serde_error(val, e))?;
        Ok(TaskDetails(val))
    }
}

impl FromStr for TaskDetailsPatch {
    type Err = ParsingError;
    fn from_str(val: &str) -> ParsingResult<Self> {
        let val = serde_json::from_str(val).map_err(|e| ParsingError::from_serde_error(val, e))?;
        Ok(TaskDetailsPatch(val))
    }
}

impl From<TaskDetails> for Details {
    fn from(src: TaskDetails) -> Self {
        src.0
    }
}

impl From<TaskDetailsPatch> for DetailsPatch {
    fn from(src: TaskDetailsPatch) -> Self {
        src.0
    }
}

pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Debug, Error, Diagnostic)]
#[error("malformed json provided")]
pub struct ParsingError {
    #[source]
    _cause: serde_json::Error,
    #[source_code]
    _input: String,
    #[label("(cause)")]
    _location: SourceOffset,
}

impl ParsingError {
    fn from_serde_error(input: impl Into<String>, cause: serde_json::Error) -> Self {
        let input = input.into();
        let location = SourceOffset::from_location(&input, cause.line(), cause.column());
        Self {
            _cause: cause,
            _input: input,
            _location: location,
        }
    }
}
