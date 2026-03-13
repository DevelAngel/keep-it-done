use crate::{TaskDetails, TaskId, TaskInfos, Uuid};

#[cfg(any(feature = "rpc"))]
use kid_types_derive::GeneratePatch;

#[cfg(feature = "ssr")]
use kid_types_derive::Patchable;

use chrono::{DateTime, FixedOffset, Offset, TimeZone, Timelike, Utc};
use derive_more::Display;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "rpc")]
use serde_with::rust::double_option;

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub struct Task {
    #[serde(flatten)]
    info: Infos,
    #[serde(flatten)]
    details: Details,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub struct Infos {
    summary: String,
    status: Status,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rpc", derive(GeneratePatch))]
#[cfg_attr(feature = "ssr", derive(Patchable))]
#[cfg_attr(feature = "ssr", patch_type(DetailsPatch))]
pub struct Details {
    priority: Option<Priority>,
    due_date: Option<DateEstimation>,
    start_date: Option<DateEstimation>,
    time_estimate: Option<TimeEstimation>,
    context: Option<String>,
    notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub enum Status {
    ToDo { since: DateTime<FixedOffset> },
    Done { since: DateTime<FixedOffset> },
}

impl Status {
    fn now() -> DateTime<FixedOffset> {
        Utc::now().with_nanosecond(0).unwrap().fixed_offset()
    }
}

impl Default for Status {
    fn default() -> Self {
        let since = Self::now();
        Self::ToDo { since }
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StatusVariant {
            StructLike(SLStatus),
            UnitLike(ULStatus),
        }

        #[derive(Deserialize)]
        pub enum SLStatus {
            ToDo { since: DateTime<FixedOffset> },
            Done { since: DateTime<FixedOffset> },
        }

        // Be backward compatible
        #[derive(Deserialize)]
        enum ULStatus {
            ToDo,
            Done,
        }

        let status = StatusVariant::deserialize(deserializer)?;
        match status {
            StatusVariant::StructLike(SLStatus::ToDo { since }) => Ok(Status::ToDo { since }),
            StatusVariant::StructLike(SLStatus::Done { since }) => Ok(Status::Done { since }),
            StatusVariant::UnitLike(ULStatus::ToDo) => Ok(Status::ToDo { since: Self::now() }),
            StatusVariant::UnitLike(ULStatus::Done) => Ok(Status::Done { since: Self::now() }),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToDo { since } => {
                let since = since.to_rfc2822();
                write!(f, "todo since {since}")
            }
            Self::Done { since } => {
                let since = since.to_rfc2822();
                write!(f, "done since {since}")
            }
        }
    }
}

#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[display(rename_all = "uppercase")]
pub enum Priority {
    A,
    B,
    #[default]
    C,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub enum DateEstimation {
    Guess(String),
    Precise(DateTime<FixedOffset>),
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum DateEstimationRef<'a, Tz: TimeZone> {
    Guess(Cow<'a, str>),
    Precise(Cow<'a, DateTime<Tz>>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub enum TimeEstimation {
    Guess(String),
    Precise(Duration),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeEstimationRef<'a> {
    Guess(Cow<'a, str>),
    Precise(Cow<'a, Duration>),
}

impl<'a, Task> TaskId<'a> for (Uuid, Task) {
    fn id(&'a self) -> &'a Uuid {
        &self.0
    }
}

impl<'a> TaskId<'a> for Uuid {
    fn id(&'a self) -> &'a Uuid {
        self
    }
}

impl<'a> TaskInfos<'a> for (Uuid, Infos) {
    fn summary(&'a self) -> Cow<'a, str> {
        self.1.summary()
    }
    fn rename<T: ToString>(&'a mut self, summary: T) {
        self.1.rename(summary);
    }
    fn status(&'a self) -> &'a Status {
        self.1.status()
    }
    fn change_status(&'a mut self, status: Status) {
        self.1.change_status(status);
    }
}

impl<'a> TaskInfos<'a> for Task {
    fn summary(&'a self) -> Cow<'a, str> {
        self.info.summary()
    }
    fn rename<T: ToString>(&'a mut self, summary: T) {
        self.info.rename(summary);
    }
    fn status(&'a self) -> &'a Status {
        self.info.status()
    }
    fn change_status(&'a mut self, status: Status) {
        self.info.change_status(status);
    }
}

impl<'a> TaskInfos<'a> for Infos {
    fn summary(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(&self.summary)
    }
    fn rename<T: ToString>(&'a mut self, summary: T) {
        self.summary = summary.to_string();
    }
    fn status(&'a self) -> &'a Status {
        &self.status
    }
    fn change_status(&'a mut self, status: Status) {
        self.status = status;
    }
}

impl<'a> TaskDetails<'a> for (Uuid, Details) {
    fn priority(&'a self) -> Option<&'a Priority> {
        self.1.priority()
    }
    fn set_priority(&'a mut self, priority: Priority) {
        self.1.set_priority(priority);
    }
    fn clear_priority(&'a mut self) {
        self.1.clear_priority();
    }
    fn due_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<DateEstimationRef<'a, Tz>> {
        self.1.due_date(tz)
    }
    fn set_due_date(&'a mut self, due_date: DateEstimation) {
        self.1.set_due_date(due_date);
    }
    fn clear_due_date(&'a mut self) {
        self.1.clear_due_date();
    }
    fn start_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<DateEstimationRef<'a, Tz>> {
        self.1.start_date(tz)
    }
    fn set_start_date(&'a mut self, start_date: DateEstimation) {
        self.1.set_start_date(start_date);
    }
    fn clear_start_date(&'a mut self) {
        self.1.clear_start_date();
    }
    fn time_estimate(&'a self) -> Option<TimeEstimationRef<'a>> {
        self.1.time_estimate()
    }
    fn set_time_estimate(&'a mut self, time: TimeEstimation) {
        self.1.set_time_estimate(time);
    }
    fn clear_time_estimate(&'a mut self) {
        self.1.clear_time_estimate();
    }
    fn context(&'a self) -> Option<Cow<'a, str>> {
        self.1.context()
    }
    fn set_context<T: ToString>(&'a mut self, text: T) {
        self.1.set_context(text);
    }
    fn clear_context(&'a mut self) {
        self.1.clear_context();
    }
    fn notes(&'a self) -> Option<Cow<'a, str>> {
        self.1.notes()
    }
    fn set_notes<T: ToString>(&'a mut self, text: T) {
        self.1.set_notes(text);
    }
    fn clear_notes(&'a mut self) {
        self.1.clear_notes();
    }
}

impl<'a> TaskDetails<'a> for Task {
    fn priority(&'a self) -> Option<&'a Priority> {
        self.details.priority()
    }
    fn set_priority(&'a mut self, priority: Priority) {
        self.details.set_priority(priority);
    }
    fn clear_priority(&'a mut self) {
        self.details.clear_priority();
    }
    fn due_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<DateEstimationRef<'a, Tz>> {
        self.details.due_date(tz)
    }
    fn set_due_date(&'a mut self, due_date: DateEstimation) {
        self.details.set_due_date(due_date);
    }
    fn clear_due_date(&'a mut self) {
        self.details.clear_due_date();
    }
    fn start_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<DateEstimationRef<'a, Tz>> {
        self.details.start_date(tz)
    }
    fn set_start_date(&'a mut self, start_date: DateEstimation) {
        self.details.set_start_date(start_date);
    }
    fn clear_start_date(&'a mut self) {
        self.details.clear_start_date();
    }
    fn time_estimate(&'a self) -> Option<TimeEstimationRef<'a>> {
        self.details.time_estimate()
    }
    fn set_time_estimate(&'a mut self, time: TimeEstimation) {
        self.details.set_time_estimate(time);
    }
    fn clear_time_estimate(&'a mut self) {
        self.details.clear_time_estimate();
    }
    fn context(&'a self) -> Option<Cow<'a, str>> {
        self.details.context()
    }
    fn set_context<T: ToString>(&'a mut self, text: T) {
        self.details.set_context(text);
    }
    fn clear_context(&'a mut self) {
        self.details.clear_context();
    }
    fn notes(&'a self) -> Option<Cow<'a, str>> {
        self.details.notes()
    }
    fn set_notes<T: ToString>(&'a mut self, text: T) {
        self.details.set_notes(text);
    }
    fn clear_notes(&'a mut self) {
        self.details.clear_notes();
    }
}

impl<'a> TaskDetails<'a> for Details {
    fn priority(&'a self) -> Option<&'a Priority> {
        self.priority.as_ref()
    }
    fn set_priority(&'a mut self, priority: Priority) {
        self.priority = Some(priority)
    }
    fn clear_priority(&'a mut self) {
        self.priority = None;
    }
    fn due_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<DateEstimationRef<'a, Tz>> {
        self.due_date.as_ref().map(|date| date.as_deref(tz))
    }
    fn set_due_date(&'a mut self, due_date: DateEstimation) {
        self.due_date = Some(due_date);
    }
    fn clear_due_date(&'a mut self) {
        self.due_date = None;
    }
    fn start_date<Tz: TimeZone>(&'a self, tz: &Tz) -> Option<DateEstimationRef<'a, Tz>> {
        self.start_date.as_ref().map(|date| date.as_deref(tz))
    }
    fn set_start_date(&'a mut self, start_date: DateEstimation) {
        self.start_date = Some(start_date);
    }
    fn clear_start_date(&'a mut self) {
        self.start_date = None;
    }
    fn time_estimate(&'a self) -> Option<TimeEstimationRef<'a>> {
        self.time_estimate.as_ref().map(|time| time.as_deref())
    }
    fn set_time_estimate(&'a mut self, time: TimeEstimation) {
        self.time_estimate = Some(time);
    }
    fn clear_time_estimate(&'a mut self) {
        self.time_estimate = None;
    }
    fn context(&'a self) -> Option<Cow<'a, str>> {
        self.context.as_deref().map(Cow::Borrowed)
    }
    fn set_context<T: ToString>(&'a mut self, text: T) {
        self.context = Some(text.to_string());
    }
    fn clear_context(&'a mut self) {
        self.context = None;
    }
    fn notes(&'a self) -> Option<Cow<'a, str>> {
        self.notes.as_deref().map(Cow::Borrowed)
    }
    fn set_notes<T: ToString>(&'a mut self, text: T) {
        self.notes = Some(text.to_string());
    }
    fn clear_notes(&'a mut self) {
        self.notes = None;
    }
}

impl<'a> DateEstimation {
    fn as_deref<Tz: TimeZone>(&'a self, tz: &Tz) -> DateEstimationRef<'a, Tz> {
        match self {
            Self::Guess(s) => DateEstimationRef::Guess(Cow::Borrowed(s)),
            Self::Precise(d) => DateEstimationRef::Precise(Cow::Owned(d.with_timezone(tz))),
        }
    }
}

impl<'a> TimeEstimation {
    fn as_deref(&'a self) -> TimeEstimationRef<'a> {
        match self {
            Self::Guess(s) => TimeEstimationRef::Guess(Cow::Borrowed(s)),
            Self::Precise(t) => TimeEstimationRef::Precise(Cow::Borrowed(t)),
        }
    }
}

impl<'a, Tz: TimeZone> DateEstimationRef<'a, Tz> {
    pub fn into_owned(self) -> DateEstimation {
        match self {
            Self::Guess(s) => DateEstimation::Guess(s.into_owned()),
            Self::Precise(d) => DateEstimation::Precise(d.with_timezone(&d.offset().fix())),
        }
    }
}

impl<'a> TimeEstimationRef<'a> {
    pub fn into_owned(self) -> TimeEstimation {
        match self {
            TimeEstimationRef::Guess(s) => TimeEstimation::Guess(s.into_owned()),
            TimeEstimationRef::Precise(t) => TimeEstimation::Precise(t.into_owned()),
        }
    }
}

impl<'a, Tz: TimeZone> Display for DateEstimationRef<'a, Tz> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Guess(s) => write!(f, "{s}"),
            Self::Precise(d) => {
                let d = d.to_rfc2822();
                write!(f, "{d}")
            }
        }
    }
}

impl<'a> Display for TimeEstimationRef<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Guess(s) => write!(f, "{s}"),
            Self::Precise(d) => {
                let secs = d.as_secs();
                let (mins, secs) = (secs / 60, secs % 60);
                let (hours, mins) = (mins / 60, mins % 60);
                let (days, hours) = (hours / 24, hours % 24);
                match (days, hours, mins, secs) {
                    (0, 0, 0, _) => write!(f, "{secs} seconds"),
                    (0, 0, _, 0) => write!(f, "{mins} minutes"),
                    (0, _, 0, 0) => write!(f, "{hours} hours"),
                    (_, 0, 0, 0) => write!(f, "{days} days"),
                    (0, _, _, _) => write!(f, "{hours}:{mins:02}:{secs:02} hours"),
                    (_, _, _, _) => write!(f, "{days} days, {hours}:{mins:02}:{secs:02} hours"),
                }
            }
        }
    }
}

impl Task {
    pub fn new<T: ToString>(summary: T) -> Self {
        let info = Infos::new(summary);
        let details = Details::default();
        Self { info, details }
    }

    pub fn with_details(mut self, details: Details) -> Self {
        self.details = details;
        self
    }

    pub fn info(&self) -> &Infos {
        &self.info
    }

    pub fn details(&self) -> &Details {
        &self.details
    }

    pub fn set_details(&mut self, details: Details) {
        self.details = details;
    }

    #[cfg(feature = "ssr")]
    pub fn patch_details(&mut self, details: DetailsPatch) {
        self.details.apply_patch(details);
    }
}

impl Infos {
    fn new<T: ToString>(summary: T) -> Self {
        Self {
            summary: summary.to_string(),
            status: Status::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::SecondsFormat;

    const SUMMARY: &str = "ABC";
    const TIME_Z: &str = "2026-03-13T20:00:00Z";

    // STATUS - SERIALIZE

    #[test]
    fn serialize_status_todo() {
        let since = DateTime::parse_from_rfc3339(TIME_Z).unwrap();
        let status = Status::ToDo { since };
        let status = serde_json::to_string(&status).expect("serialization");
        assert_eq!(status, format!(r#"{{"ToDo":{{"since":"{TIME_Z}"}}}}"#));
    }

    #[test]
    fn serialize_status_done() {
        let since = DateTime::parse_from_rfc3339(TIME_Z).unwrap();
        let status = Status::Done { since };
        let status = serde_json::to_string(&status).expect("serialization");
        assert_eq!(status, format!(r#"{{"Done":{{"since":"{TIME_Z}"}}}}"#));
    }

    // STATUS - DESERIALIZE

    #[test]
    fn deserialize_status_todo() {
        let status = format!(r#"{{"ToDo":{{"since":"{TIME_Z}"}}}}"#);
        let status: Status = serde_json::from_str(&status).expect("deserialization");
        let since = DateTime::parse_from_rfc3339(TIME_Z).unwrap();
        assert_eq!(status, Status::ToDo { since });
    }

    #[test]
    fn deserialize_status_done() {
        let status = format!(r#"{{"Done":{{"since":"{TIME_Z}"}}}}"#);
        let status: Status = serde_json::from_str(&status).expect("deserialization");
        let since = DateTime::parse_from_rfc3339(TIME_Z).unwrap();
        assert_eq!(status, Status::Done { since });
    }

    #[test]
    fn deserialize_status_todo_without_since() {
        let status = format!(r#""ToDo""#);
        let status: Status = serde_json::from_str(&status).expect("deserialization");
        let since = Status::now(); // maybe instable
        assert_eq!(status, Status::ToDo { since });
    }

    #[test]
    fn deserialize_status_done_without_since() {
        let status = format!(r#""Done""#);
        let status: Status = serde_json::from_str(&status).expect("deserialization");
        let since = Status::now(); // maybe instable
        assert_eq!(status, Status::Done { since });
    }

    // TASK - SERIALIZE

    #[test]
    fn serialize_minimal_task_todo() {
        let task = Task::new(SUMMARY);
        let task = serde_json::to_string(&task).expect("serialization");
        let since = Status::now(); // maybe instable
        let since = since.to_rfc3339_opts(SecondsFormat::Secs, true);
        let task_expected =
            format!(r#"{{"summary":"{SUMMARY}","status":{{"ToDo":{{"since":"{since}"}}}}}}"#);
        assert_eq!(task, task_expected);
    }

    // TASK - DESERIALIZE

    #[test]
    fn deserialize_minimal_task_todo() {
        let task =
            format!(r#"{{"summary":"{SUMMARY}","status":{{"ToDo":{{"since":"{TIME_Z}"}}}}}}"#);
        let task: Task = serde_json::from_str(&task).expect("deserialization");
        assert!(matches!(
            task,
            Task {
                info: Infos {
                    summary: _,
                    status: Status::ToDo { since: _ },
                },
                details: Details { .. }
            }
        ));
    }

    #[test]
    fn deserialize_minimal_task_done() {
        let task =
            format!(r#"{{"summary":"{SUMMARY}","status":{{"Done":{{"since":"{TIME_Z}"}}}}}}"#);
        let task: Task = serde_json::from_str(&task).expect("deserialization");
        assert!(matches!(
            task,
            Task {
                info: Infos {
                    summary: _,
                    status: Status::Done { since: _ },
                },
                details: Details { .. }
            }
        ));
    }

    #[test]
    fn deserialize_minimal_task_todo_without_since() {
        let task = format!(r#"{{"summary":"{SUMMARY}","status":"ToDo"}}"#);
        let task: Task = serde_json::from_str(&task).expect("deserialization");
        assert!(matches!(
            task,
            Task {
                info: Infos {
                    summary: _,
                    status: Status::ToDo { since: _ },
                },
                details: Details { .. }
            }
        ));
    }

    #[test]
    fn deserialize_minimal_task_done_without_since() {
        let task = format!(r#"{{"summary":"{SUMMARY}","status":"Done"}}"#);
        let task: Task = serde_json::from_str(&task).expect("deserialization");
        assert!(matches!(
            task,
            Task {
                info: Infos {
                    summary: _,
                    status: Status::Done { since: _ },
                },
                details: Details { .. }
            }
        ));
    }
}
