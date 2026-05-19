use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::given;
use tarpc::context;

use std::collections::HashMap;

use crate::seeds;
use crate::world::AppWorld;

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

#[given("no tasks at all")]
async fn empty_task_list(world: &mut AppWorld) -> Result<()> {
    let dir = world.tasks_dir.as_ref().expect("tasks_dir must be set");
    world
        .rpc
        .switch_dir(context::current(), dir.path().to_path_buf())
        .await
        .expect("RPC call failed")
        .expect("switch_dir failed");
    let count = world.rpc.count(context::current()).await?;
    assert_eq!(count, 0);
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
            ),
            "done" => seeds::write_done(dir.path(), summary, category, days_ago),
            other => panic!("unknown status: {other}"),
        }
    }

    // Now switch the server to this directory so it loads the files.
    world
        .rpc
        .switch_dir(context::current(), dir.path().to_path_buf())
        .await
        .expect("RPC call failed")
        .expect("switch_dir failed");

    Ok(())
}
