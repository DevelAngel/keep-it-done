use anyhow::Result;
use chrono::{Datelike, TimeDelta, Utc};
use cucumber::gherkin::Step;
use cucumber::given;
use std::collections::HashMap;

use crate::helpers::TEST_CONTROL_ADDR;
use crate::seeds;
use crate::world::AppWorld;

fn parse_weekday(s: &str) -> chrono::Weekday {
    use chrono::Weekday::*;
    match s.to_ascii_lowercase().as_str() {
        "monday" | "mon" => Mon,
        "tuesday" | "tue" => Tue,
        "wednesday" | "wed" => Wed,
        "thursday" | "thu" => Thu,
        "friday" | "fri" => Fri,
        "saturday" | "sat" => Sat,
        "sunday" | "sun" => Sun,
        _ => panic!("invalid weekday: {s}"),
    }
}

#[given(expr = "I am logged in as {string}")]
async fn logged_in_as(world: &mut AppWorld, user: String) -> Result<()> {
    let mut headers = HashMap::new();
    headers.insert("Remote-User".into(), user.into());
    world.http.cdp().network().set_extra_http_headers(headers).await?;
    Ok(())
}

#[given("no user is logged in")]
async fn no_user_logged_in(world: &mut AppWorld) -> Result<()> {
    let headers = HashMap::new();
    world.http.cdp().network().set_extra_http_headers(headers).await?;
    Ok(())
}

#[given(expr = "a viewport of {int} by {int}")]
async fn set_viewport(world: &mut AppWorld, width: u32, height: u32) -> Result<()> {
    world
        .http
        .set_window_rect(0, 0, width, height)
        .await?;
    Ok(())
}

#[given(expr = "today is simulated as {word}")]
async fn simulate_weekday(world: &mut AppWorld, weekday: String) -> Result<()> {
    let target = parse_weekday(&weekday);
    let current = Utc::now().date_naive().weekday();

    let days = (target.num_days_from_monday() as i64
        - current.num_days_from_monday() as i64
        + 7)
        % 7;
    let offset_seconds = days * 86_400;

    world
        .admin
        .post(format!("http://{TEST_CONTROL_ADDR}/set-time-offset"))
        .json(&serde_json::json!({ "seconds": offset_seconds }))
        .send()
        .await?
        .error_for_status()
        .expect("set_time_offset failed");
    world.time_offset_seconds = Some(offset_seconds);

    Ok(())
}

#[given("no tasks at all")]
async fn empty_task_list(world: &mut AppWorld) -> Result<()> {
    let dir = world.tasks_dir.as_ref().expect("tasks_dir must be set");
    world
        .admin
        .post(format!("http://{TEST_CONTROL_ADDR}/switch-dir"))
        .json(&serde_json::json!({ "dir": dir.path() }))
        .send()
        .await?
        .error_for_status()
        .expect("switch_dir failed");
    let count: serde_json::Value = world
        .admin
        .get(format!("http://{TEST_CONTROL_ADDR}/count"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(count["count"], 0);
    Ok(())
}

#[given("the following tasks")]
async fn create_tasks(world: &mut AppWorld, step: &Step) -> Result<()> {
    let dir = world.tasks_dir.as_ref().expect("tasks_dir must be set");
    let table = step.table.as_ref().expect("data table required");
    let headers = &table.rows[0];

    for row in &table.rows[1..] {
        let col = |name: &str| -> Option<&str> {
            headers
                .iter()
                .position(|h| h == name)
                .map(|idx| row[idx].trim())
                .filter(|val| !val.is_empty())
        };

        let summary = col("summary").expect("summary required");
        let category = col("category").expect("category required");
        let status = col("status").expect("status required");
        let days_ago: u64 = col("days ago")
            .expect("days ago required")
            .parse()
            .expect("days ago must be a number");

        let offset = world.time_offset_seconds.unwrap_or(0);
        let reference = Utc::now() + TimeDelta::seconds(offset);
        match status {
            "open" => seeds::write_open(
                dir.path(),
                summary,
                category,
                col("context").unwrap_or(""),
                col("estimate"),
                col("priority"),
                col("start"),
                col("due"),
                col("note"),
                days_ago,
                reference,
            ),
            "done" => seeds::write_done(dir.path(), summary, category, days_ago, reference),
            other => panic!("unknown status: {other}"),
        }
    }

    // Now switch the server to this directory so it loads the files.
    world
        .admin
        .post(format!("http://{TEST_CONTROL_ADDR}/switch-dir"))
        .json(&serde_json::json!({ "dir": dir.path() }))
        .send()
        .await?
        .error_for_status()
        .expect("switch_dir failed");

    Ok(())
}
