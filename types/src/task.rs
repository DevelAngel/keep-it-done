use crate::{TaskDetails, TaskId, TaskInfos, Uuid};

use chrono::{DateTime, FixedOffset, Offset, TimeZone};
use derive_more::Display;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub struct Details {
    priority: Option<Priority>,
    due_date: Option<DateEstimation>,
    start_date: Option<DateEstimation>,
    time_estimate: Option<TimeEstimation>,
    context: Option<String>,
    notes: Option<String>,
}

#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema, clap::ValueEnum))]
#[cfg_attr(feature = "cli", clap(rename_all = "lowercase"))]
#[display(rename_all = "lowercase")]
pub enum Status {
    #[default]
    ToDo,
    Done,
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
}

impl Infos {
    fn new<T: ToString>(summary: T) -> Self {
        Self {
            summary: summary.to_string(),
            status: Status::default(),
        }
    }
}
