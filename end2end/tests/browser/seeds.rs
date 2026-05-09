use kid_cli::TaskServiceClient;
use kid_types::{Task, TaskDate, TaskDetails, TaskInfos, TaskPriority, TaskTimeEstimate, Utc};

use chrono::TimeDelta;
use indexmap::IndexSet;
use tarpc::context;
use uuid::{NoContext, Timestamp, Uuid};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ACTOR: &str = "e2e-seed";

/// Build a task and send it to the server with a backdated UUID.
pub async fn add_open(
    rpc: &TaskServiceClient,
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
    send(rpc, task, days_ago).await;
}

/// Build a completed task and send it to the server.
pub async fn add_done(
    rpc: &TaskServiceClient,
    summary: &str,
    category: &str,
    days_ago: u64,
) {
    let mut task = build(summary, category, "", None, None, None, None, None);
    task.mark_done();
    send(rpc, task, days_ago).await;
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

async fn send(rpc: &TaskServiceClient, task: Task, days_ago: u64) {
    let id = uuid_days_ago(days_ago);
    rpc.add_with_id(context::current(), id, task, ACTOR.to_owned())
        .await
        .expect("RPC add_with_id failed");
}

/// Create a UUID v7 with a timestamp `days` days in the past.
fn uuid_days_ago(days: u64) -> Uuid {
    let now = SystemTime::now() - Duration::from_secs(days * 86_400);
    let d = now.duration_since(UNIX_EPOCH).unwrap();
    let ts = Timestamp::from_unix(NoContext, d.as_secs(), d.subsec_nanos());
    Uuid::new_v7(ts)
}
