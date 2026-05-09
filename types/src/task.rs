use crate::{TaskDetails, TaskId, TaskInfos, Uuid};

#[cfg(any(feature = "rpc"))]
use kid_types_derive::GeneratePatch;

#[cfg(feature = "ssr")]
use kid_types_derive::Patchable;

use chrono::{DateTime, FixedOffset, Timelike, Utc};
use derive_more::Display;
use indexmap::{IndexMap, IndexSet};
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::EnumIter;

#[cfg(feature = "rpc")]
use serde_with::rust::double_option;

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Filter {
    Todo,
    Done,
    HasTimeEstimate,
    RecentlyChanged,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "cli", schemars(transparent))]
pub struct Category(String);

impl Default for Category {
    fn default() -> Self {
        Self("Inbox".to_string())
    }
}

impl FromStr for Category {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("category must not be empty")
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl Deref for Category {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Category {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.is_empty() {
            Err(serde::de::Error::custom("category must not be empty"))
        } else {
            Ok(Self(s))
        }
    }
}

/// Lenient deserializer for the `Infos.category` field: maps the empty string
/// to `Category::default()` so that legacy files with `"context": ""` load
/// without error. Use only at the field level — never for user-facing input.
fn deserialize_category_lenient<'de, D: Deserializer<'de>>(d: D) -> Result<Category, D::Error> {
    let s = String::deserialize(d)?;
    if s.is_empty() { Ok(Category::default()) } else { Ok(Category(s)) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "cli", schemars(transparent))]
pub struct Summary(String);

impl FromStr for Summary {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("summary must not be empty")
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl Deref for Summary {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Summary {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Summary {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if s.is_empty() {
            Err(serde::de::Error::custom("summary must not be empty"))
        } else {
            Ok(Self(s))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "cli", schemars(transparent))]
pub struct Context(String);

impl FromStr for Context {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("context must not be empty")
        } else if !s.starts_with('@') {
            Err("context must start with '@'")
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl Deref for Context {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Context {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Context {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub struct Task {
    #[serde(flatten)]
    info: Infos,
    #[serde(flatten)]
    details: Details,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    #[cfg_attr(feature = "cli", schemars(with = "IndexMap<String, Vec<DateTime<FixedOffset>>>"))]
    authors: IndexMap<String, Vec<DateTime<FixedOffset>>>,
}

/// Snapshot of task authors with their edit timestamps.
///
/// Produced from the internal `IndexMap` storage; intended for
/// transfer across the server-function boundary.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Authors(Vec<(String, DateTime<FixedOffset>)>);

impl std::ops::Deref for Authors {
    type Target = Vec<(String, DateTime<FixedOffset>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&IndexMap<String, Vec<DateTime<FixedOffset>>>> for Authors {
    fn from(map: &IndexMap<String, Vec<DateTime<FixedOffset>>>) -> Self {
        let mut entries: Vec<_> = map.iter()
            .flat_map(|(name, timestamps)| {
                timestamps.iter().map(move |ts| (name.clone(), *ts))
            })
            .collect();
        entries.sort_by(|(_, a), (_, b)| b.cmp(a));
        Self(entries)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub struct Infos {
    summary: Summary,
    status: Status,
    #[serde(alias = "context", default, deserialize_with = "deserialize_category_lenient")]
    category: Category,
    #[serde(default, skip_serializing_if = "IndexSet::is_empty")]
    #[cfg_attr(feature = "cli", schemars(with = "Vec<Context>"))]
    contexts: IndexSet<Context>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    priority: Option<Priority>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rpc", derive(GeneratePatch))]
#[cfg_attr(feature = "ssr", derive(Patchable))]
#[cfg_attr(feature = "ssr", patch_type(DetailsPatch))]
pub struct Details {
    #[serde(default, deserialize_with = "deserialize_due_date")]
    due_date: Option<Date>,
    #[serde(default, deserialize_with = "deserialize_due_date")]
    start_date: Option<Date>,
    time_estimate: Option<TimeEstimate>,
    #[serde(default, skip_serializing_if = "Availability::is_default")]
    availability: Availability,
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

#[derive(Clone, Copy, Debug, Default, Display, Serialize, Deserialize, PartialEq, Eq, EnumIter)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
#[display(rename_all = "uppercase")]
pub enum Priority {
    A,
    B,
    #[default]
    C,
}

impl FromStr for Priority {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            _ => Err("priority must be A, B, or C"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub struct Date {
    pub date: DateTime<FixedOffset>,
    pub soft: bool,
}

fn deserialize_due_date<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Date>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Current(Date),
        Legacy(LegacyDate),
    }
    #[derive(Deserialize)]
    enum LegacyDate {
        Guess(String),
        Precise(DateTime<FixedOffset>),
    }
    let maybe: Option<Raw> = Option::deserialize(d)?;
    Ok(maybe.and_then(|raw| match raw {
        Raw::Current(dd) => Some(dd),
        Raw::Legacy(LegacyDate::Precise(date)) => Some(Date { date, soft: false }),
        Raw::Legacy(LegacyDate::Guess(s)) => {
            // Try to parse as a datetime with timezone offset
            DateTime::parse_from_rfc3339(&s)
                .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%z"))
                .map(|date| Date { date, soft: true })
                .ok()
                // Fall back to date-only (midnight UTC)
                .or_else(|| {
                    chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|ndt| Date { date: ndt.and_utc().fixed_offset(), soft: true })
                })
        }
    }))
}


#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub enum TimeEstimate {
    Min15,
    Min30,
    Min45,
    Hours1,
    Hours2,
    HalfDay,
    Day1,
    Day2,
}

impl From<Duration> for TimeEstimate {
    fn from(time: Duration) -> Self {
        let time = time.as_secs() / 60u64;
        match time {
            0..20 => Self::Min15,
            20..35 => Self::Min30,
            35..50 => Self::Min45,
            50..70 => Self::Hours1,
            70..135 => Self::Hours2,
            135..740 => Self::HalfDay,
            740..1500 => Self::Day1,
            _ => Self::Day2,
        }
    }
}

impl<'de> Deserialize<'de> for TimeEstimate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Current(Current),
            Legacy(Legacy),
        }

        #[derive(Deserialize)]
        enum Current {
            Min15, Min30, Min45, Hours1, Hours2, HalfDay, Day1, Day2,
        }

        #[derive(Deserialize)]
        enum Legacy {
            Guess(String),
            Precise(Duration),
        }

        Ok(match Raw::deserialize(deserializer)? {
            Raw::Current(c) => match c {
                Current::Min15   => Self::Min15,
                Current::Min30   => Self::Min30,
                Current::Min45   => Self::Min45,
                Current::Hours1  => Self::Hours1,
                Current::Hours2  => Self::Hours2,
                Current::HalfDay => Self::HalfDay,
                Current::Day1    => Self::Day1,
                Current::Day2    => Self::Day2,
            },
            Raw::Legacy(Legacy::Guess(s)) => {
                s.parse().unwrap_or(Self::Day2)
            }
            Raw::Legacy(Legacy::Precise(d)) => d.into(),
        })
    }
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
    fn summary(&'a self) -> &'a str {
        self.1.summary()
    }
    fn rename(&'a mut self, summary: Summary) {
        self.1.rename(summary);
    }
    fn status(&'a self) -> &'a Status {
        self.1.status()
    }
    fn change_status(&'a mut self, status: Status) {
        self.1.change_status(status);
    }
    fn category(&'a self) -> &'a str {
        self.1.category()
    }
    fn set_category(&'a mut self, category: Category) {
        self.1.set_category(category);
    }
    fn contexts(&'a self) -> &'a IndexSet<Context> {
        self.1.contexts()
    }
    fn set_contexts(&'a mut self, contexts: IndexSet<Context>) {
        self.1.set_contexts(contexts);
    }
    fn extend_contexts(&'a mut self, contexts: IndexSet<Context>) {
        self.1.extend_contexts(contexts);
    }
    fn priority(&'a self) -> Option<&'a Priority> {
        self.1.priority()
    }
    fn set_priority(&'a mut self, priority: Priority) {
        self.1.set_priority(priority);
    }
    fn clear_priority(&'a mut self) {
        self.1.clear_priority();
    }
}

impl<'a> TaskInfos<'a> for Task {
    fn summary(&'a self) -> &'a str {
        self.info.summary()
    }
    fn rename(&'a mut self, summary: Summary) {
        self.info.rename(summary);
    }
    fn status(&'a self) -> &'a Status {
        self.info.status()
    }
    fn change_status(&'a mut self, status: Status) {
        self.info.change_status(status);
    }
    fn category(&'a self) -> &'a str {
        self.info.category()
    }
    fn set_category(&'a mut self, category: Category) {
        self.info.set_category(category);
    }
    fn contexts(&'a self) -> &'a IndexSet<Context> {
        self.info.contexts()
    }
    fn set_contexts(&'a mut self, contexts: IndexSet<Context>) {
        self.info.set_contexts(contexts);
    }
    fn extend_contexts(&'a mut self, contexts: IndexSet<Context>) {
        self.info.extend_contexts(contexts);
    }
    fn priority(&'a self) -> Option<&'a Priority> {
        self.info.priority()
    }
    fn set_priority(&'a mut self, priority: Priority) {
        self.info.set_priority(priority);
    }
    fn clear_priority(&'a mut self) {
        self.info.clear_priority();
    }
}

impl<'a> TaskInfos<'a> for Infos {
    fn summary(&'a self) -> &'a str {
        &self.summary
    }
    fn rename(&'a mut self, summary: Summary) {
        self.summary = summary;
    }
    fn status(&'a self) -> &'a Status {
        &self.status
    }
    fn change_status(&'a mut self, status: Status) {
        self.status = status;
    }
    fn category(&'a self) -> &'a str {
        &self.category
    }
    fn set_category(&'a mut self, category: Category) {
        self.category = category;
    }
    fn contexts(&'a self) -> &'a IndexSet<Context> {
        &self.contexts
    }
    fn set_contexts(&'a mut self, contexts: IndexSet<Context>) {
        self.contexts = contexts;
    }
    fn extend_contexts(&'a mut self, contexts: IndexSet<Context>) {
        self.contexts.extend(contexts);
    }
    fn priority(&'a self) -> Option<&'a Priority> {
        self.priority.as_ref()
    }
    fn set_priority(&'a mut self, priority: Priority) {
        self.priority = Some(priority);
    }
    fn clear_priority(&'a mut self) {
        self.priority = None;
    }
}

impl<'a> TaskDetails<'a> for (Uuid, Details) {
    fn due_date(&'a self) -> Option<&'a Date> {
        self.1.due_date()
    }
    fn set_due_date(&'a mut self, due_date: Date) {
        self.1.set_due_date(due_date);
    }
    fn clear_due_date(&'a mut self) {
        self.1.clear_due_date();
    }
    fn start_date(&'a self) -> Option<&'a Date> {
        self.1.start_date()
    }
    fn set_start_date(&'a mut self, date: Date) {
        self.1.set_start_date(date);
    }
    fn clear_start_date(&'a mut self) {
        self.1.clear_start_date();
    }
    fn time_estimate(&'a self) -> Option<&'a TimeEstimate> {
        self.1.time_estimate()
    }
    fn set_time_estimate(&mut self, time: TimeEstimate) {
        self.1.set_time_estimate(time);
    }
    fn clear_time_estimate(&mut self) {
        self.1.clear_time_estimate();
    }
    fn availability(&'a self) -> &'a Availability {
        self.1.availability()
    }
    fn set_availability(&'a mut self, availability: Availability) {
        self.1.set_availability(availability);
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
    fn due_date(&'a self) -> Option<&'a Date> {
        self.details.due_date()
    }
    fn set_due_date(&'a mut self, due_date: Date) {
        self.details.set_due_date(due_date);
    }
    fn clear_due_date(&'a mut self) {
        self.details.clear_due_date();
    }
    fn start_date(&'a self) -> Option<&'a Date> {
        self.details.start_date()
    }
    fn set_start_date(&'a mut self, date: Date) {
        self.details.set_start_date(date);
    }
    fn clear_start_date(&'a mut self) {
        self.details.clear_start_date();
    }
    fn time_estimate(&'a self) -> Option<&'a TimeEstimate> {
        self.details.time_estimate()
    }
    fn set_time_estimate(&mut self, time: TimeEstimate) {
        self.details.set_time_estimate(time);
    }
    fn clear_time_estimate(&mut self) {
        self.details.clear_time_estimate();
    }
    fn availability(&'a self) -> &'a Availability {
        &self.details.availability
    }
    fn set_availability(&'a mut self, availability: Availability) {
        self.details.set_availability(availability);
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
    fn due_date(&'a self) -> Option<&'a Date> {
        self.due_date.as_ref()
    }
    fn set_due_date(&'a mut self, due_date: Date) {
        self.due_date = Some(due_date);
    }
    fn clear_due_date(&'a mut self) {
        self.due_date = None;
    }
    fn start_date(&'a self) -> Option<&'a Date> {
        self.start_date.as_ref()
    }
    fn set_start_date(&'a mut self, date: Date) {
        self.start_date = Some(date);
    }
    fn clear_start_date(&'a mut self) {
        self.start_date = None;
    }
    fn time_estimate(&'a self) -> Option<&'a TimeEstimate> {
        self.time_estimate.as_ref()
    }
    fn set_time_estimate(&mut self, time: TimeEstimate) {
        self.time_estimate = Some(time);
    }
    fn clear_time_estimate(&mut self) {
        self.time_estimate = None;
    }
    fn availability(&'a self) -> &'a Availability {
        &self.availability
    }
    fn set_availability(&'a mut self, availability: Availability) {
        self.availability = availability;
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

impl FromStr for TimeEstimate {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use strum::IntoEnumIterator;
        let lower = s.trim().to_lowercase();
        Self::iter()
            .find(|v| lower == v.short_label() || lower == v.to_string().to_lowercase())
            .ok_or("unknown time estimate")
    }
}

impl TimeEstimate {
    /// `short_label` has an exhaustive match — adding a variant breaks the build
    /// until updated. Use `TimeEstimate::iter()` (strum) to iterate all variants.
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Min15   => "15m",
            Self::Min30   => "30m",
            Self::Min45   => "45m",
            Self::Hours1  => "1h",
            Self::Hours2  => "2h",
            Self::HalfDay => "½d",
            Self::Day1    => "1d",
            Self::Day2    => "2d",
        }
    }

    /// Calendar days needed before the due date to complete this task.
    ///
    /// Sub-day estimates return 0 (completable on the due date itself).
    /// Used by the attention-date computation to determine lead time.
    pub fn lead_days(self) -> u32 {
        match self {
            Self::Min15 | Self::Min30 | Self::Min45
            | Self::Hours1 | Self::Hours2 | Self::HalfDay => 0,
            Self::Day1 => 1,
            Self::Day2 => 2,
        }
    }
}

impl Display for TimeEstimate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Min15   => write!(f, "15 minutes"),
            Self::Min30   => write!(f, "30 minutes"),
            Self::Min45   => write!(f, "45 minutes"),
            Self::Hours1  => write!(f, "1 hour"),
            Self::Hours2  => write!(f, "2 hours"),
            Self::HalfDay => write!(f, "½ day"),
            Self::Day1    => write!(f, "1 day"),
            Self::Day2    => write!(f, "2 days"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(schemars::JsonSchema))]
pub enum Availability {
    #[default]
    Anytime,
    WeekdayOnly,
    WeekendOnly,
}

impl Availability {
    fn is_default(&self) -> bool {
        matches!(self, Self::Anytime)
    }

    /// Whether a calendar date is eligible for work under this constraint.
    pub fn is_eligible(self, date: chrono::NaiveDate) -> bool {
        use chrono::{Datelike, Weekday};
        match self {
            Self::Anytime => true,
            Self::WeekdayOnly => !matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
            Self::WeekendOnly => matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
        }
    }
}

impl Task {
    pub fn new(summary: Summary) -> Self {
        let info = Infos::new(summary);
        let details = Details::default();
        Self { info, details, authors: IndexMap::new() }
    }

    pub fn with_category(mut self, category: Category) -> Self {
        self.info.category = category;
        self
    }

    pub fn with_contexts(mut self, contexts: IndexSet<Context>) -> Self {
        self.info.contexts = contexts;
        self
    }

    pub fn with_details(mut self, details: Details) -> Self {
        self.details = details;
        self
    }

    pub fn add_author(&mut self, actor: impl Into<String>) {
        let now = Utc::now().with_nanosecond(0).unwrap().fixed_offset();
        let timestamps = self.authors.entry(actor.into()).or_default();

        // Debounce: if the author's last edit was < 5 min ago,
        // update it in place instead of appending a duplicate.
        match timestamps.last_mut() {
            Some(last) if (now - *last).num_minutes() < 5 => *last = now,
            _ => timestamps.push(now),
        }
    }

    pub fn authors(&self) -> &IndexMap<String, Vec<DateTime<FixedOffset>>> {
        &self.authors
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
    fn new(summary: Summary) -> Self {
        Self {
            summary,
            status: Status::default(),
            category: Category::default(),
            contexts: IndexSet::new(),
            priority: None,
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
        let task = Task::new(SUMMARY.parse().unwrap());
        let task = serde_json::to_string(&task).expect("serialization");
        let since = Status::now(); // maybe instable
        let since = since.to_rfc3339_opts(SecondsFormat::Secs, true);
        let task_expected =
            format!(r#"{{"summary":"{SUMMARY}","status":{{"ToDo":{{"since":"{since}"}}}},"category":"{}"}}"#, Category::default());
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
                    ..
                },
                details: Details { .. },
                ..
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
                    ..
                },
                details: Details { .. },
                ..
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
                    ..
                },
                details: Details { .. },
                ..
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
                    ..
                },
                details: Details { .. },
                ..
            }
        ));
    }

    // TIME ESTIMATE - SERIALIZE / DESERIALIZE

    #[test]
    fn serialize_time_estimate() {
        let v = serde_json::to_string(&TimeEstimate::Min30).unwrap();
        assert_eq!(v, r#""Min30""#);
    }

    #[test]
    fn deserialize_time_estimate_current() {
        let v: TimeEstimate = serde_json::from_str(r#""HalfDay""#).unwrap();
        assert_eq!(v, TimeEstimate::HalfDay);
    }

    #[test]
    fn deserialize_time_estimate_legacy_precise_nearest() {
        let v: TimeEstimate =
            serde_json::from_str(r#"{"Precise":{"secs":2880,"nanos":0}}"#).unwrap();
        assert_eq!(v, TimeEstimate::Min45);
    }

    #[test]
    fn deserialize_time_estimate_legacy_guess_unparseable() {
        let v: TimeEstimate = serde_json::from_str(r#"{"Guess":"a weekend"}"#).unwrap();
        assert_eq!(v, TimeEstimate::Day2);
    }

    #[test]
    fn deserialize_time_estimate_legacy_guess_short_label() {
        let v: TimeEstimate = serde_json::from_str(r#"{"Guess":"1h"}"#).unwrap();
        assert_eq!(v, TimeEstimate::Hours1);
    }

    #[test]
    fn deserialize_time_estimate_legacy_guess_display() {
        let v: TimeEstimate = serde_json::from_str(r#"{"Guess":"30 minutes"}"#).unwrap();
        assert_eq!(v, TimeEstimate::Min30);
    }

    // DATE - DESERIALIZE (migration)

    #[test]
    fn deserialize_due_date_legacy_precise() {
        let task: Task = serde_json::from_str(&format!(
            r#"{{"summary":"X","status":"ToDo","due_date":{{"Precise":"{TIME_Z}"}}}}"#
        )).unwrap();
        let date = task.details.due_date.unwrap();
        assert_eq!(date.date, DateTime::parse_from_rfc3339(TIME_Z).unwrap());
        assert!(!date.soft);
    }

    #[test]
    fn deserialize_due_date_legacy_guess_unparseable() {
        let task: Task = serde_json::from_str(
            r#"{"summary":"X","status":"ToDo","due_date":{"Guess":"next week"}}"#
        ).unwrap();
        assert!(task.details.due_date.is_none());
    }

    #[test]
    fn deserialize_due_date_legacy_guess_rfc3339() {
        let task: Task = serde_json::from_str(&format!(
            r#"{{"summary":"X","status":"ToDo","due_date":{{"Guess":"{TIME_Z}"}}}}"#
        )).unwrap();
        let date = task.details.due_date.unwrap();
        assert_eq!(date.date, DateTime::parse_from_rfc3339(TIME_Z).unwrap());
        assert!(date.soft);
    }

    #[test]
    fn deserialize_due_date_legacy_guess_date_only() {
        let task: Task = serde_json::from_str(
            r#"{"summary":"X","status":"ToDo","due_date":{"Guess":"2026-04-30"}}"#
        ).unwrap();
        let date = task.details.due_date.unwrap();
        assert_eq!(date.date, DateTime::parse_from_rfc3339("2026-04-30T00:00:00+00:00").unwrap());
        assert!(date.soft);
    }

    #[test]
    fn deserialize_start_date_legacy_precise() {
        let task: Task = serde_json::from_str(&format!(
            r#"{{"summary":"X","status":"ToDo","start_date":{{"Precise":"{TIME_Z}"}}}}"#
        )).unwrap();
        let date = task.details.start_date.unwrap();
        assert_eq!(date.date, DateTime::parse_from_rfc3339(TIME_Z).unwrap());
        assert!(!date.soft);
    }

    #[test]
    fn deserialize_start_date_legacy_guess_unparseable() {
        let task: Task = serde_json::from_str(
            r#"{"summary":"X","status":"ToDo","start_date":{"Guess":"end of month"}}"#
        ).unwrap();
        assert!(task.details.start_date.is_none());
    }

    #[test]
    fn deserialize_start_date_legacy_guess_date_only() {
        let task: Task = serde_json::from_str(
            r#"{"summary":"X","status":"ToDo","start_date":{"Guess":"2026-07-01"}}"#
        ).unwrap();
        let date = task.details.start_date.unwrap();
        assert_eq!(date.date, DateTime::parse_from_rfc3339("2026-07-01T00:00:00+00:00").unwrap());
        assert!(date.soft);
    }

    // TIME ESTIMATE - LEAD DAYS

    #[test]
    fn lead_days_sub_day_estimates_are_zero() {
        use strum::IntoEnumIterator;
        for v in TimeEstimate::iter() {
            match v {
                TimeEstimate::Day1 | TimeEstimate::Day2 => continue,
                other => assert_eq!(other.lead_days(), 0, "{other}"),
            }
        }
    }

    #[test]
    fn lead_days_day1() {
        assert_eq!(TimeEstimate::Day1.lead_days(), 1);
    }

    #[test]
    fn lead_days_day2() {
        assert_eq!(TimeEstimate::Day2.lead_days(), 2);
    }

    // AVAILABILITY - SERIALIZE / DESERIALIZE

    #[test]
    fn serialize_availability_weekday_only() {
        let v = serde_json::to_string(&Availability::WeekdayOnly).unwrap();
        assert_eq!(v, r#""WeekdayOnly""#);
    }

    #[test]
    fn serialize_availability_weekend_only() {
        let v = serde_json::to_string(&Availability::WeekendOnly).unwrap();
        assert_eq!(v, r#""WeekendOnly""#);
    }

    #[test]
    fn deserialize_availability_all_variants() {
        assert_eq!(
            serde_json::from_str::<Availability>(r#""Anytime""#).unwrap(),
            Availability::Anytime,
        );
        assert_eq!(
            serde_json::from_str::<Availability>(r#""WeekdayOnly""#).unwrap(),
            Availability::WeekdayOnly,
        );
        assert_eq!(
            serde_json::from_str::<Availability>(r#""WeekendOnly""#).unwrap(),
            Availability::WeekendOnly,
        );
    }

    #[test]
    fn availability_default_is_anytime() {
        assert_eq!(Availability::default(), Availability::Anytime);
    }

    // AVAILABILITY - TASK ROUND-TRIP

    #[test]
    fn task_without_availability_deserializes_to_anytime() {
        let task: Task = serde_json::from_str(
            r#"{"summary":"X","status":"ToDo"}"#
        ).unwrap();
        assert_eq!(task.details.availability, Availability::Anytime);
    }

    #[test]
    fn task_with_availability_round_trips() {
        let task: Task = serde_json::from_str(
            r#"{"summary":"X","status":"ToDo","availability":"WeekendOnly"}"#
        ).unwrap();
        assert_eq!(task.details.availability, Availability::WeekendOnly);

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains(r#""availability":"WeekendOnly""#));
    }

    #[test]
    fn task_with_anytime_skips_serialization() {
        let task = Task::new(SUMMARY.parse().unwrap());
        assert_eq!(task.details.availability, Availability::Anytime);

        let json = serde_json::to_string(&task).unwrap();
        assert!(!json.contains("availability"));
    }
}
