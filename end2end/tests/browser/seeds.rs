use kid_types::{Task, TaskDate, TaskDetails, TaskInfos, TaskPriority, TaskTimeEstimate, Utc};

use chrono::{SecondsFormat, Timelike, TimeDelta};
use indexmap::IndexSet;
use uuid::{NoContext, Timestamp, Uuid};

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ACTOR: &str = "e2e-seed";

/// Build an open task and write it as a JSON file with backdated timestamps.
pub fn write_open(
    dir: &Path,
    summary: &str,
    category: &str,
    context: &str,
    estimate: Option<&str>,
    priority: Option<&str>,
    start: Option<&str>,
    due: Option<&str>,
    note: Option<&str>,
    days_ago: u64,
) {
    let task = build(summary, category, context, estimate, priority, start, due, note);
    write_file(dir, task, days_ago);
}

/// Build a completed task and write it as a JSON file.
pub fn write_done(
    dir: &Path,
    summary: &str,
    category: &str,
    days_ago: u64,
) {
    let mut task = build(summary, category, "", None, None, None, None, None);
    task.mark_done();
    write_file(dir, task, days_ago);
}

fn build(
    summary: &str,
    category: &str,
    context: &str,
    estimate: Option<&str>,
    priority: Option<&str>,
    start: Option<&str>,
    due: Option<&str>,
    note: Option<&str>,
) -> Task {
    let ctx: IndexSet<_> = if context.is_empty() {
        IndexSet::new()
    } else {
        [context.parse().unwrap()].into()
    };
    let mut t = Task::new(summary.parse().unwrap())
        .with_category(category.parse().unwrap())
        .with_contexts(ctx);
    if let Some(e) = estimate {
        t.set_time_estimate(e.parse::<TaskTimeEstimate>().unwrap());
    }
    if let Some(p) = priority {
        t.set_priority(p.parse::<TaskPriority>().unwrap());
    }
    if let Some(s) = start {
        t.set_start_date(date_from_relative_days(s));
    }
    if let Some(d) = due {
        t.set_due_date(date_from_relative_days(d));
    }
    if let Some(n) = note {
        t.set_notes(n);
    }
    t
}

/// Parse a relative day offset (`+5`, `-3`, `0`) into a [`TaskDate`].
fn date_from_relative_days(s: &str) -> TaskDate {
    let days: i64 = s.parse().unwrap_or_else(|_| panic!("invalid day offset: {s}"));
    let date = Utc::now().fixed_offset() + TimeDelta::days(days);
    TaskDate { date, soft: false }
}

/// Serialize the task to JSON, backdate timestamps, and write to disk.
///
/// Both `status.since` and the `authors` entry are set to the
/// timestamp implied by `days_ago`, giving the Recent Changes view
/// realistic, spread-out modification times.
fn write_file(dir: &Path, task: Task, days_ago: u64) {
    let id = uuid_days_ago(days_ago);
    let since = datetime_days_ago(days_ago);
    let since_str = since.to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut json = serde_json::to_value(&task).expect("task serialization");

    // Backdate status.since
    if let Some(status) = json.get_mut("status") {
        for variant in ["ToDo", "Done"] {
            if let Some(inner) = status.get_mut(variant) {
                inner["since"] = serde_json::json!(since_str);
            }
        }
    }

    // Set authors with backdated timestamp
    json["authors"] = serde_json::json!({ ACTOR: [since_str] });

    let path = dir.join(format!("task-{}.json", id.as_simple()));
    std::fs::write(path, serde_json::to_string_pretty(&json).unwrap())
        .expect("write task file");
}

/// Create a UUID v7 with a timestamp `days` days in the past.
fn uuid_days_ago(days: u64) -> Uuid {
    let now = SystemTime::now() - Duration::from_secs(days * 86_400);
    let d = now.duration_since(UNIX_EPOCH).unwrap();
    let ts = Timestamp::from_unix(NoContext, d.as_secs(), d.subsec_nanos());
    Uuid::new_v7(ts)
}

/// UTC timestamp `days` days in the past, nanoseconds zeroed.
fn datetime_days_ago(days: u64) -> chrono::DateTime<chrono::FixedOffset> {
    let offset = TimeDelta::days(days as i64);
    (Utc::now() - offset)
        .with_nanosecond(0)
        .unwrap()
        .fixed_offset()
}
